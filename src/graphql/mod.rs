pub mod links;

use std::sync::{Arc, Mutex};
use std::time::Duration;
use async_graphql::*;
use futures_util::stream::Stream;
use mongodb::Client;
use mongodb::options::{ClientOptions, ResolverConfig};
use async_graphql::{Subscription, Object};
use fred::clients::RedisClient;
use fred::interfaces::ClientLike;
use fred::prelude::{ReconnectPolicy, RedisConfig};

use crate::graphql::links::{LinkMutations, LinkQueries, LinkSubscriptions};

#[derive(Default)]
pub struct QueryRoot;

#[derive(Default)]
pub struct MutationRoot;
#[derive(Default)]
pub struct SubscriptionRoot;

#[derive(MergedObject, Default)]
pub struct Queries(QueryRoot, LinkQueries);

#[derive(MergedObject, Default)]
pub struct Mutations(MutationRoot, LinkMutations);

/*#[derive(MergedObject, Default)]
pub struct Subscriptions(SubscriptionRoot, LinkSubscriptions);*/


pub type GraphqlSchema = Schema<Queries, Mutations, LinkSubscriptions>;


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
  let mut publish_client = RedisClient::new(config.clone());
  let _ = publish_client.connect(Some(policy.clone()));
  let _ = publish_client.wait_for_connect().await;

  let mut subscribe_client = RedisClient::new(config.clone());
  let _ = subscribe_client.connect(Some(policy.clone()));
  let _ = subscribe_client.wait_for_connect().await;


  Schema::build(Queries::default(), Mutations::default(), LinkSubscriptions)
    .data(mongo_database)
    .data(redis_client)
    .data(PubSub { publish_client, subscribe_client })
    .finish()
}
