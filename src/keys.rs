use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use serde::{Deserialize, Serialize};
use serenity::client::bridge::gateway::ShardManager;
use serenity::prelude::TypeMapKey;
use tokio::sync::Mutex;

pub struct ShardManagerContainer;
impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

pub struct BotCtl;

impl TypeMapKey for BotCtl {
    type Value = AtomicBool;
}

pub struct Data {
    pub(crate) stock: u64,
    pub(crate) image_url: String,
}

#[derive(Serialize, Deserialize)]
pub struct Component {
    pub name: String,
    pub lcsc: String,
    pub enabled: bool,
    pub channel_id: u64,
    pub prev_stock: u64,
    pub role_id: u64,
}

#[derive(Serialize, Deserialize)]
pub struct Components {
    pub components: Vec<Component>,
}
