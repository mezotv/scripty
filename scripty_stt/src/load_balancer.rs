use std::{
	net::SocketAddr,
	sync::{
		atomic::{AtomicBool, AtomicUsize, Ordering},
		Arc,
	},
	time::Duration,
};

use dashmap::DashMap;
use once_cell::sync::OnceCell;
use scripty_config::SttServiceDefinition;
use tokio::{
	io,
	io::{AsyncReadExt, AsyncWriteExt},
	net::lookup_host,
};

use crate::{ModelError, Stream, NUM_STT_SERVICE_TRIES};

pub static LOAD_BALANCER: OnceCell<LoadBalancer> = OnceCell::new();

/// Round-robin load balancer that equally loads all tasks,
/// until one notes that it is overloaded, at which point it is removed from the pool.
///
/// If it notifies the master that it is no longer overloaded, it is re-added.
pub struct LoadBalancer {
	/// The current worker index.
	current_index: AtomicUsize,
	/// A list of all workers.
	workers:       DashMap<usize, LoadBalancedStream>,
}

impl LoadBalancer {
	pub async fn new() -> io::Result<Self> {
		let stt_services = scripty_config::get_config().stt_services.clone();
		let mut peer_addresses: Vec<SocketAddr> = Vec::new();
		for service in stt_services {
			match service {
				SttServiceDefinition::HostString(host) => peer_addresses.extend(
					lookup_host(host)
						.await
						.expect("Could not resolve stt hostname"),
				),
				SttServiceDefinition::IPTuple(addr, port) => peer_addresses.push(SocketAddr::new(
					addr.parse()
						.expect("Could not parse IP address for stt server"),
					port,
				)),
			}
		}

		let workers = DashMap::new();
		for (n, addr) in peer_addresses.into_iter().enumerate() {
			workers.insert(n, LoadBalancedStream::new(addr).await?);
		}
		Ok(Self {
			current_index: AtomicUsize::new(0),
			workers,
		})
	}

	fn get_next_worker_idx(&self) -> usize {
		self.current_index
			.fetch_update(Ordering::Release, Ordering::Acquire, |x| {
				if x == self.workers.len() {
					Some(0)
				} else {
					Some(x + 1)
				}
			})
			.expect("get_next_worker_idx::{closure} should never return None")
	}

	fn find_worker(&self) -> Result<usize, ModelError> {
		let mut idx = self.get_next_worker_idx();
		let mut iter_count: usize = 0;
		let mut allow_overload = false;

		loop {
			if let Some(worker) = self.workers.get(&idx) {
				// if we're allowing overloading, or this worker isn't overloaded and isn't in error
				if (allow_overload && worker.can_overload)
					|| !worker.is_overloaded() && !worker.is_in_error()
				{
					// usually this is going to be the fast path and it will immediately return this worker
					// if it isn't, this is still decently fast, an O(2n) operation worst case
					// given there's very likely never going to be more than 255 workers, this is fine
					return Ok(idx);
				}
			}

			idx = self.get_next_worker_idx();

			// are we back at the start?
			if !allow_overload && iter_count > self.workers.len() {
				// we've looped through all workers and none are available:
				// try again, but this time allow overloading
				allow_overload = true;
			}

			iter_count += 1;

			if iter_count > NUM_STT_SERVICE_TRIES {
				// failed to find any available workers
				// give up and return an error
				scripty_metrics::get_metrics()
					.stt_server_fetch_failure
					.inc_by(1);
				error!(
					"no available STT servers after {} tries",
					NUM_STT_SERVICE_TRIES
				);
				return Err(ModelError::NoAvailableServers);
			}
		}
	}

	pub async fn get_stream(&self, language: &str, verbose: bool) -> Result<Stream, ModelError> {
		let worker_id = self.find_worker()?;
		let worker = self.workers.get(&worker_id).expect("worker should exist");

		let metrics = scripty_metrics::get_metrics();
		match worker.open_connection(language, verbose).await {
			Ok(s) => {
				metrics.stt_server_fetch_success.inc_by(1);
				Ok(s)
			}
			Err(e) => {
				metrics.stt_server_fetch_failure.inc_by(1);
				Err(e)
			}
		}
	}
}

pub struct LoadBalancedStream {
	peer_address:  SocketAddr,
	is_overloaded: Arc<AtomicBool>,
	can_overload:  bool,
	is_in_error:   Arc<AtomicBool>,
}

impl LoadBalancedStream {
	#[inline]
	pub fn is_overloaded(&self) -> bool {
		self.is_overloaded.load(Ordering::Relaxed)
	}

	#[inline]
	pub fn is_in_error(&self) -> bool {
		self.is_in_error.load(Ordering::Relaxed)
	}

	pub(crate) async fn open_connection(
		&self,
		language: &str,
		verbose: bool,
	) -> Result<Stream, ModelError> {
		if !self.can_overload && self.is_overloaded() {
			return Err(ModelError::Io(io::Error::new(
				io::ErrorKind::Other,
				"remote is overloaded",
			)));
		}

		let res = Stream::new(language, verbose, self.peer_address).await;
		self.is_in_error.store(res.is_err(), Ordering::Relaxed);
		res
	}

	pub async fn new(peer_address: SocketAddr) -> io::Result<Self> {
		// open a connection to the remote
		let mut peer_stream = tokio::net::TcpStream::connect(peer_address).await?;

		// convert this connection into a data-only connection (send 0x04)
		peer_stream.write_u8(0x04).await?;

		// wait for a response of 0x06 (status connection open, fields max_utilization: f64, can_overload: bool)
		if peer_stream.read_u8().await? != 0x06 {
			return Err(io::Error::new(
				io::ErrorKind::Other,
				"unexpected response from server",
			));
		}

		// read the fields
		let max_utilization = peer_stream.read_f64().await?;
		let can_overload = peer_stream.read_u8().await? == 1;

		debug!(
			?max_utilization,
			?can_overload,
			?peer_address,
			"got data for new stream"
		);

		let is_overloaded = Arc::new(AtomicBool::new(false));
		let iso2 = Arc::clone(&is_overloaded);
		let is_in_error = Arc::new(AtomicBool::new(false));
		let iie2 = Arc::clone(&is_in_error);

		// spawn a background task that will monitor the connection, and if it reports being overloaded, sets the overloaded flag
		tokio::spawn(async move {
			let metrics = scripty_metrics::get_metrics();
			let mut peer_stream = peer_stream;
			loop {
				let data: u8 = tokio::select! {
					data_type = peer_stream.read_u8() => {
						match data_type {
							Ok(d) => d,
							Err(e) => {
								error!(?peer_address, "error reading from peer: {}", e);
								// try to reconnect
								peer_stream = match tokio::net::TcpStream::connect(peer_address).await {
									Ok(s) => s,
									Err(e) => {
										error!(?peer_address, "error reconnecting to peer: {}", e);
										iie2.store(true, Ordering::Relaxed);
										metrics.stt_server_fetch_failure.inc_by(1);
										const ONE_SECOND: Duration = Duration::from_secs(1);
										tokio::time::sleep(ONE_SECOND).await;
										continue;
									}
								};
								continue;
							}
						}
					},
					_ = tokio::signal::ctrl_c() => {
						break
					}
				};
				iie2.store(false, Ordering::Relaxed);

				if data != 0x07 {
					error!(?peer_address, "unexpected data type from peer: {}", data);
					// toss the error to the handler which will retry
					continue;
				}
				metrics.stt_server_fetch_success.inc_by(1);

				// read payload (utilization: f64)
				let utilization = match peer_stream.read_f64().await {
					Ok(u) => u,
					Err(e) => {
						error!(?peer_address, "error reading from peer: {}", e);
						// toss the error to the handler which will try to reconnect or exit
						continue;
					}
				};

				// if the utilization is above the threshold, set the overloaded flag
				iso2.store(utilization > max_utilization, Ordering::Relaxed);
			}
			// write 0x03 to the stream to close the connection
			if let Err(e) = peer_stream.write_u8(0x03).await {
				error!(
					?peer_address,
					"error closing connection to {}: {}", peer_address, e
				);
			}
		});

		Ok(Self {
			peer_address,
			is_overloaded,
			can_overload,
			is_in_error,
		})
	}
}