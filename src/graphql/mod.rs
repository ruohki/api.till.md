pub mod admin;
pub mod channel;
pub mod guards;
pub mod roles;
pub mod user;

use std::convert::From;

use crate::graphql::admin::AdminMutations;
use crate::graphql::channel::{ChannelMutations, ChannelQueries, ChannelSubscriptions};
use crate::graphql::user::{UserMutations, UserQueries};

use async_graphql::*;

use futures_util::stream::Stream;
use mongodb::bson::oid::ObjectId;

use mongodb::{Database};
use std::time::Duration;

use std::env::var;
use std::sync::Arc;
use lazy_static::lazy_static;
use crate::connections::PubSub;
use crate::ModelFor;
use crate::models::channel::ChannelEntity;
use crate::models::user::UserEntity;

lazy_static! {
    static ref MONGO_URL: String = var("MONGO_URL").expect("MONGO_URL not set in environment");
    static ref REDIS_URL: String = var("REDIS_URL").expect("REDIS_URL not set in environment");
}

pub trait FromOid {
  fn from_object_id(_: ObjectId) -> Self;
}

impl FromOid for ID {
  fn from_object_id(oid: ObjectId) -> Self {
    ID::from(oid.to_hex())
  }
}

#[derive(Default, Debug)]
pub struct QueryRoot;

#[derive(Default)]
pub struct MutationRoot;

#[derive(Default)]
pub struct SubscriptionRoot;

#[derive(MergedObject, Default)]
pub struct Queries(/*QueryRoot,*/ UserQueries, ChannelQueries);

#[derive(MergedObject, Default)]
pub struct Mutations(
/*  MutationRoot,*/
  AdminMutations,
  ChannelMutations,
  UserMutations,
);

#[derive(MergedSubscription, Default)]
pub struct Subscriptions(SubscriptionRoot, ChannelSubscriptions);

pub type GraphqlSchema = Schema<Queries, Mutations, Subscriptions>;

#[Subscription]
impl SubscriptionRoot {
  async fn interval_dummy(&self, #[graphql(default = 1)] n: i32) -> impl Stream<Item=i32> {
    let mut value = 0;
    async_stream::stream! {
          loop {
            futures_timer::Delay::new(Duration::from_secs(1)).await;
            value += n;
            yield value;
          }
        }
  }
}

pub async fn build_schema(db: Database, pubsub: PubSub) -> GraphqlSchema {
  Schema::build(
    Queries::default(),
    Mutations::default(),
    Subscriptions::default(),
  )
  .data(pubsub)
  // Model
  .data(ModelFor::<UserEntity>::new(
    Arc::new(db.clone()),
    "users",
  ))
  .data(ModelFor::<ChannelEntity>::new(
    Arc::new(db.clone()),
    "channel",
  ))
  .finish()
}
