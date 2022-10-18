use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct LinkModel {
  #[serde(rename = "_id")]
  pub id: Option<ObjectId>,

  pub url: String,
  pub label: String
}

impl LinkModel {
  pub fn new(url: String, label: String) -> Self {
    Self {
      id: Some(ObjectId::new()),
      url,
      label
    }
  }
}