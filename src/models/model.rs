use std::sync::Arc;
use mongodb::bson::Document;
use mongodb::{error::Result, Collection, Database, Cursor};
use mongodb::options::{DeleteOptions, FindOneOptions, FindOptions, UpdateModifications, UpdateOptions};
use mongodb::results::{DeleteResult, UpdateResult};
use serde::de::DeserializeOwned;
use crate::models::user::UserEntity;

#[derive(Clone)]
pub struct ModelFor<T> {
  _collection: Collection<T>
}


impl<T> ModelFor<T>
  where T: Unpin + DeserializeOwned + Send + Sync {

  pub fn new(db: Arc<Database>, collection_name: String) -> Self {
    Self {
      _collection: db.collection::<T>(collection_name.as_str())
    }
  }

  pub async fn find_one(&self, filter: impl Into<Option<Document>>, options: impl Into<Option<FindOneOptions>>) -> Result<Option<T>> {
    self._collection.find_one(filter, options).await
  }

  pub async fn find(&self, filter: impl Into<Option<Document>>, options: impl Into<Option<FindOptions>>) -> Result<Cursor<T>> {
    self._collection.find(filter, options).await
  }

  pub async fn update_one(&self, filter: Document, update: impl Into<UpdateModifications>, options: impl Into<Option<UpdateOptions>>) -> Result<UpdateResult> {
    self._collection.update_one(filter, update, options).await
  }

  pub async fn delete_one(&self, filter: Document, options: impl Into<Option<DeleteOptions>>) -> Result<DeleteResult> {
    self._collection.delete_one(filter, options).await
  }
}