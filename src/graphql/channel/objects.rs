use async_graphql::{ID, SimpleObject};
use serde::{Deserialize, Serialize};
use crate::graphql::links::Link;
use crate::models::channel::ChannelEntity;
use crate::models::links::LinkModel;

#[derive(Default, Clone, SimpleObject, Deserialize, Serialize)]
pub struct Channel {
  pub id: ID,

  pub name: String,
  pub description: String,
  pub public: bool,

  pub when_created: u64,
  pub last_publish: u64,
  pub last_subscribe: u64,
}

impl From<ChannelEntity> for Channel {
  fn from(e: ChannelEntity) -> Self {
    Channel {
      id: ID::from(e.id.unwrap().to_hex()),
      name: e.name,
      description: e.description,
      public: e.public,
      when_created: e.when_created,
      last_publish: e.last_publish,
      last_subscribe: e.last_subscribe
    }
  }
}