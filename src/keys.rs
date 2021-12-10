use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use chrono::{DateTime, Utc};
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

pub struct Uptime;
impl TypeMapKey for Uptime {
    type Value = HashMap<String, DateTime<Utc>>;
}

pub struct Data {
    pub stock: i64,
    pub image_url: String,
}

#[derive(Serialize, Deserialize)]
pub struct Component {
    pub name: String,
    pub lcsc: String,
    pub enabled: bool,
    pub channel_id: u64,
    pub prev_stock: i64,
    pub role_id: u64,
}

#[derive(Serialize, Deserialize)]
pub struct Components {
    pub components: Vec<Component>,
}

#[derive(Serialize, Deserialize)]
pub struct Role {
    pub role_name: String,
    pub role_id: u64
}

#[derive(Serialize, Deserialize)]
pub struct Roles {
    pub roles: Vec<Role>,
}



