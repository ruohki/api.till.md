use async_graphql::{ID, SimpleObject};
use serde::{Deserialize, Serialize};
use crate::graphql::user::objects::User;
use crate::models::channel::ChannelEntity;

#[derive(Default, Clone, SimpleObject, Deserialize, Serialize)]
pub struct Channel {
  pub id: ID,

  pub name: String,
  pub description: String,
  pub public: bool,

  pub when_created: i64,
  pub last_publish: i64,
  pub last_subscribe: i64,
}

impl From<ChannelEntity> for Channel {
  fn from(e: ChannelEntity) -> Self {
    Channel {
      id: ID::from(e.id.unwrap().to_hex()),
      name: e.name,
      description: e.description,
      public: e.public,
      when_created: e.when_created.timestamp_millis(),
      last_publish: e.last_publish.timestamp_millis(),
      last_subscribe: e.last_subscribe.timestamp_millis()
    }
  }
}

#[derive(Clone, SimpleObject, Serialize, Deserialize)]
pub struct ChannelMessage {
  pub id: ID,
  pub message: String,
  pub send_when: i64,
  pub send_from: User,
  pub send_to: Channel,
}