use serenity::gateway::ShardManager;
use serenity::prelude::TypeMapKey;
use std::sync::Arc;
use tokio::sync::Mutex;

#[macro_use]
extern crate tracing;

mod block_in_place;
mod embed_pagination;
mod hash_user_id;
mod hex_vec;
pub mod latency;
mod separate_num;
mod humanize_duration;

pub use humanize_duration::humanize_duration;
pub use block_in_place::block_in_place;
pub use embed_pagination::do_paginate;
pub use hash_user_id::hash_user_id;
pub use hex_vec::vec_to_hex;
pub use separate_num::separate_num;

pub struct ShardManagerWrapper;
impl TypeMapKey for ShardManagerWrapper {
    type Value = Arc<Mutex<ShardManager>>;
}
