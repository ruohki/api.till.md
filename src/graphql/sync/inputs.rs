use async_graphql::*;
use serde::{Deserialize, Serialize};
use crate::graphql::sync::objects::Stat;

#[derive(Enum, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum ObjectType {
  File,
  Folder
}

#[derive(InputObject)]
pub struct StatArgs {
  pub ctime: f64,
  pub mtime: f64,
  pub size: usize
}

#[derive(InputObject)]
pub struct CreateArgs {
  pub path: String,
  pub name: String,
  pub extension: Option<String>,
  pub object_type: ObjectType,
  pub stat: Option<StatArgs>
}