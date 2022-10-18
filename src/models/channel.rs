use std::time::{SystemTime, UNIX_EPOCH};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ChannelEntity {
  #[serde(rename = "_id")]
  pub id: Option<ObjectId>,

  pub name: String,
  pub description: String,
  pub public: bool,

  pub when_created: u64,
  pub last_publish: u64,
  pub last_subscribe: u64,
}

impl ChannelEntity {
  pub fn new(name: String, description: String, public: bool) -> Self {
    Self {
      id: Some(ObjectId::new()),
      name,
      description,
      public,
      when_created: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
      last_publish: 0,
      last_subscribe: 0,
    }
  }
}