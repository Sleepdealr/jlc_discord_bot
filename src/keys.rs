use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serenity::client::bridge::gateway::ShardManager;
use serenity::prelude::TypeMapKey;
use sqlx::PgPool;
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

#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct Component {
    pub name: String,
    pub lcsc: String,
    pub enabled: bool,
    pub channel_id: i64,
    pub prev_stock: i64,
    pub role_id: i64,
}

#[derive(Serialize, Deserialize)]
pub struct Components {
    pub components: Vec<Component>,
}

#[derive(Serialize, Deserialize)]
pub struct Role {
    pub role_name: String,
    pub role_id: u64,
}

#[derive(Serialize, Deserialize)]
pub struct Roles {
    pub roles: Vec<Role>,
}

#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct Datasheet {
    pub name: String,
    pub link: String,
}

#[derive(Serialize, Deserialize)]
pub struct Datasheets {
    pub datasheets: Vec<Datasheet>,
}

pub struct DatabasePool;
impl TypeMapKey for DatabasePool {
    type Value = PgPool;
}
