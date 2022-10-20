pub mod admin;
pub mod channel;
pub mod guards;
pub mod roles;
pub mod user;

use std::convert::From;
use std::sync::Arc;

use crate::graphql::admin::AdminMutations;
use crate::graphql::channel::{ChannelMutations, ChannelQueries, ChannelSubscriptions};
use crate::graphql::user::{UserMutations, UserQueries};
use crate::models::user::UserEntity;
use async_graphql::*;
use fred::clients::RedisClient;
use fred::interfaces::ClientLike;
use fred::prelude::{ReconnectPolicy, RedisConfig};
use futures_util::stream::Stream;
use mongodb::bson::oid::ObjectId;
use mongodb::options::{ClientOptions, ResolverConfig};
use mongodb::Client;
use std::time::Duration;

use crate::ModelFor;
use crate::models::channel::ChannelEntity;

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


pub struct PubSub {
  pub publish_client: RedisClient,
  pub subscribe_client: RedisClient,
}

pub async fn build_schema() -> GraphqlSchema {
  // MongoDB
  let options = ClientOptions::parse_with_resolver_config(
    "mongodb://localhost:27017/rust",
    ResolverConfig::cloudflare(),
  )
    .await
    .unwrap();
  let mongo_client = Client::with_options(options).unwrap();
  let mongo_database = mongo_client.default_database().unwrap();

  let redis_conn_url = "redis://localhost:6379";
  let redis_client = redis::Client::open(redis_conn_url).expect("Invalid connection URL");

  let config = RedisConfig::default();
  let policy = ReconnectPolicy::default();
  let publish_client = RedisClient::new(config.clone());
  let _ = publish_client.connect(Some(policy.clone()));
  let _ = publish_client.wait_for_connect().await;

  let subscribe_client = RedisClient::new(config.clone());
  let _ = subscribe_client.connect(Some(policy.clone()));
  let _ = subscribe_client.wait_for_connect().await;

  Schema::build(
    Queries::default(),
    Mutations::default(),
    Subscriptions::default(),
  )
    .data(redis_client)
    .data(ModelFor::<UserEntity>::new(
      Arc::new(mongo_database.clone()),
      "users",
    ))
    .data(ModelFor::<ChannelEntity>::new(
      Arc::new(mongo_database.clone()),
      "channel",
    ))
    .data(PubSub {
      publish_client,
      subscribe_client,
    })
    .finish()
}
