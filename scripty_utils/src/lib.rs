use serenity::client::bridge::gateway::ShardManager;
use serenity::prelude::TypeMapKey;
use std::sync::Arc;
use tokio::sync::Mutex;

#[macro_use]
extern crate tracing;

pub mod cpu;
pub mod latency;
mod separate_num;

pub use separate_num::separate_num;

pub struct ShardManagerWrapper;
impl TypeMapKey for ShardManagerWrapper {
    type Value = Arc<Mutex<ShardManager>>;
}
