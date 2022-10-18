pub mod links;

use std::time::Duration;
use async_graphql::*;
use futures_util::stream::Stream;
use mongodb::Client;
use mongodb::options::{ClientOptions, ResolverConfig};
use async_graphql::{Subscription, Object};

use crate::graphql::links::{LinkMutations, LinkQueries};

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


pub type GraphqlSchema = Schema<Queries, Mutations, SubscriptionRoot>;


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


pub async fn build_schema() -> GraphqlSchema {

  // A Client is needed to connect to MongoDB:
  // An extra line of code to work around a DNS issue on Windows:
  let options =
    ClientOptions::parse_with_resolver_config("mongodb://localhost:27017/rust", ResolverConfig::cloudflare())
      .await.unwrap();
  let client = Client::with_options(options).unwrap();
  let database = client.default_database().unwrap();
  // Print the databases in our MongoDB cluster:


  Schema::build(Queries::default(), Mutations::default(), SubscriptionRoot)
    .data(database)
    .finish()
}
