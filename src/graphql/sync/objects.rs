use async_graphql::*;
use graphql_int64_scalar::Int64Scalar;
use serde::{Serialize, Deserialize};
use crate::graphql::sync::inputs;
use crate::graphql::sync::inputs::StatArgs;

#[derive(SimpleObject, Serialize, Deserialize)]
pub struct Stat {
  pub ctime: f64,
  pub mtime: f64,
  pub size: usize
}

impl Stat {
  pub fn from_args(args: StatArgs) -> Self {
    Self {
      ctime: args.ctime,
      mtime: args.mtime,
      size: args.size,
    }
  }
}

#[derive(SimpleObject, Serialize, Deserialize)]
pub struct FileOperation {
  pub basename: String,
  pub name: String,
  pub extension: String,
  pub path: String,
  pub stat: Option<Stat>,
}

#[derive(SimpleObject, Serialize, Deserialize)]
pub struct PathOperation {
  pub basename: String,
  pub name: String,
  pub path: String,
  pub stat: Option<Stat>,
}

#[derive(Union, Serialize, Deserialize)]
pub enum Operation {
  File(FileOperation),
  Path(PathOperation),
}

#[derive(SimpleObject, Serialize, Deserialize)]
pub struct RenameMessage {
  pub operation_type: inputs::ObjectType,
  pub operation: Operation,
}

#[derive(SimpleObject, Serialize, Deserialize)]
pub struct CreateMessage {
  pub operation_type: inputs::ObjectType,
  pub operation: Operation,
}

#[derive(Union, Serialize, Deserialize)]
pub enum SyncEvent {
  Create(CreateMessage),
  Rename(RenameMessage),
}