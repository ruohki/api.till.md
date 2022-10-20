pub mod links;
pub mod channel;
pub mod user;
pub mod admin;
pub mod guards;
pub mod roles;

use std::convert::From;
use std::sync::Arc;

use std::time::Duration;
use async_graphql::*;
use futures_util::stream::Stream;
use mongodb::Client;
use mongodb::options::{ClientOptions, ResolverConfig};
use async_graphql::{Subscription, Object};
use fred::clients::RedisClient;
use fred::interfaces::ClientLike;
use fred::prelude::{ReconnectPolicy, RedisConfig};
use mongodb::bson::oid::ObjectId;
use crate::graphql::admin::AdminMutations;
use crate::graphql::channel::{ChannelMutations, ChannelQueries, ChannelSubscriptions};
use crate::graphql::links::{LinkMutations, LinkQueries, LinkSubscriptions};
use crate::graphql::user::{UserMutations, UserQueries};
use crate::models::user::UserEntity;

use guards::RoleGuard;
use roles::Role;
use crate::ModelFor;

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
pub struct Queries(QueryRoot, LinkQueries, UserQueries, ChannelQueries);

#[derive(MergedObject, Default)]
pub struct Mutations(MutationRoot, AdminMutations, LinkMutations, ChannelMutations, UserMutations);

#[derive(MergedSubscription, Default)]
pub struct Subscriptions(SubscriptionRoot, LinkSubscriptions, ChannelSubscriptions);

pub type GraphqlSchema = Schema<Queries, Mutations, Subscriptions>;

#[Subscription]
impl SubscriptionRoot {
  async fn interval_dummy(&self, #[graphql(default = 1)] n: i32) -> impl Stream<Item = i32> {
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

#[Object]
impl QueryRoot {
  async fn links(&self) -> LinkQueries {
    LinkQueries::default()
  }

  #[graphql(guard = "RoleGuard::new(Role::Admin)")]
  async fn test(&self, _ctx: &Context<'_>) -> String {
    "Iam guarded!".to_string()
  }
}

#[Object]
impl MutationRoot {
  async fn login(&self) -> String { todo!() }
}

pub struct PubSub {
  pub publish_client: RedisClient,
  pub subscribe_client: RedisClient
}

pub async fn build_schema() -> GraphqlSchema {
  // MongoDB
  let options =
    ClientOptions::parse_with_resolver_config("mongodb://localhost:27017/rust", ResolverConfig::cloudflare())
      .await.unwrap();
  let mongo_client = Client::with_options(options).unwrap();
  let mongo_database = mongo_client.default_database().unwrap();

  let redis_conn_url = "redis://localhost:6379";
  let redis_client = redis::Client::open(redis_conn_url)
    .expect("Invalid connection URL");


  let config = RedisConfig::default();
  let policy = ReconnectPolicy::default();
  let publish_client = RedisClient::new(config.clone());
  let _ = publish_client.connect(Some(policy.clone()));
  let _ = publish_client.wait_for_connect().await;

  let subscribe_client = RedisClient::new(config.clone());
  let _ = subscribe_client.connect(Some(policy.clone()));
  let _ = subscribe_client.wait_for_connect().await;


  let user_collection = mongo_database.collection::<UserEntity>("users");



  Schema::build(Queries::default(), Mutations::default(), Subscriptions::default())
    .data(mongo_database.clone())
    .data(user_collection)
    .data(redis_client)
    .data(ModelFor::<UserEntity>::new(Arc::new(mongo_database),"users".to_string()))
    .data(PubSub { publish_client, subscribe_client })
    .finish()
}
