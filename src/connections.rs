use fred::clients::{RedisClient};
use fred::error::RedisError;
use fred::interfaces::ClientLike;
use fred::prelude::{ReconnectPolicy, RedisConfig};
use mongodb::Client;
use mongodb::options::{ClientOptions, ResolverConfig};

#[derive(Clone)]
pub struct PubSub {
  pub publish: RedisClient,
  pub subscribe: RedisClient,
}

pub async fn build_database_connection(connection_string: &String) -> Option<mongodb::Database> {
  // MongoDB
  let options = ClientOptions::parse_with_resolver_config(
    connection_string,
    ResolverConfig::cloudflare(),
  )
    .await
    .unwrap();
  let mongo_client = Client::with_options(options).unwrap();

  mongo_client.default_database()
}

pub async fn build_pubsub_client(connection_string: &String) -> Result<PubSub, RedisError> {

  let config = RedisConfig::from_url(connection_string)?;

  let policy = ReconnectPolicy::new_exponential(0, 100, 30_000, 2);

  let publish = RedisClient::new(config.clone());
  publish.connect(Some(policy.clone()));
  publish.wait_for_connect().await?;

  let subscribe = RedisClient::new(config.clone());
  subscribe.connect(Some(policy.clone()));
  subscribe.wait_for_connect().await?;

  Ok(PubSub {
    publish,
    subscribe
  })
}