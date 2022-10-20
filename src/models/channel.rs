use chrono::Utc;
use mongodb::bson::DateTime;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelEntity {
  #[serde(rename = "_id")]
  pub id: Option<ObjectId>,

  pub name: String,
  pub description: String,
  pub public: bool,
  pub when_created: DateTime,
  pub last_publish: DateTime,
  pub last_subscribe: DateTime,
}

impl ChannelEntity {
  pub fn new(name: String, description: String, public: bool) -> Self {
    Self {
      id: Some(ObjectId::new()),
      name,
      description,
      public,
      when_created: DateTime::from_millis(Utc::now().timestamp_millis()),
      last_publish: DateTime::from_millis(Utc::now().timestamp_millis()),
      last_subscribe: DateTime::from_millis(Utc::now().timestamp_millis()),
    }
  }
}