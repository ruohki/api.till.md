use std::borrow::Borrow;

use mongodb::bson::Document;
use mongodb::options::{DeleteOptions, FindOneAndUpdateOptions, FindOneOptions, FindOptions, InsertOneOptions, UpdateModifications, UpdateOptions};
use mongodb::results::{DeleteResult, InsertOneResult, UpdateResult};
use mongodb::{error::Result, Collection, Cursor, Database};
use serde::de::DeserializeOwned;
use std::sync::Arc;
use serde::Serialize;

#[derive(Clone)]
pub struct ModelFor<T> {
  _collection: Collection<T>,
}

impl<T> ModelFor<T>
  where
    T: Unpin + DeserializeOwned + Send + Sync + Serialize,
{
  #[allow(dead_code)]
  pub fn new(db: Arc<Database>, collection_name: &str) -> Self {
    Self {
      _collection: db.collection::<T>(collection_name),
    }
  }

  #[allow(dead_code)]
  pub async fn find_one(
    &self,
    filter: impl Into<Option<Document>>,
    options: impl Into<Option<FindOneOptions>>,
  ) -> Result<Option<T>> {
    self._collection.find_one(filter, options).await
  }

  #[allow(dead_code)]
  pub async fn find(
    &self,
    filter: impl Into<Option<Document>>,
    options: impl Into<Option<FindOptions>>,
  ) -> Result<Cursor<T>> {
    self._collection.find(filter, options).await
  }

  #[allow(dead_code)]
  pub async fn find_one_and_update(
    &self, filter: Document, update: impl Into<UpdateModifications>, options: impl Into<Option<FindOneAndUpdateOptions>>,
  ) -> Result<Option<T>>
  {
    self._collection.find_one_and_update(filter, update, options).await
  }

  #[allow(dead_code)]
  pub async fn insert_one(
    &self, doc: impl Borrow<T>, options: impl Into<Option<InsertOneOptions>>
  ) -> Result<InsertOneResult>
  {
    self._collection.insert_one(doc, options).await
  }

  #[allow(dead_code)]
  pub async fn update_one(
    &self,
    filter: Document,
    update: impl Into<UpdateModifications>,
    options: impl Into<Option<UpdateOptions>>,
  ) -> Result<UpdateResult> {
    self._collection.update_one(filter, update, options).await
  }

  #[allow(dead_code)]
  pub async fn delete_one(
    &self,
    filter: Document,
    options: impl Into<Option<DeleteOptions>>,
  ) -> Result<DeleteResult> {
    self._collection.delete_one(filter, options).await
  }
}
