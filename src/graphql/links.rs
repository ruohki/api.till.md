use std::borrow::BorrowMut;
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use crate::models::links::LinkModel;
use crate::graphql::PubSub;


use async_graphql::{Context, ID, Object, InputObject, SimpleObject, Subscription, async_stream};
use async_graphql::async_stream::stream;
use async_graphql::futures_util::{Stream, StreamExt, TryStreamExt};
use fred::interfaces::PubsubInterface;
use fred::types::RedisValue;
use mongodb::{Database};
use serde::{Deserialize, Serialize};

#[derive(Default)]
pub struct LinkQueries;

#[derive(Default)]
pub struct LinkMutations;

#[derive(Default)]
pub struct LinkSubscriptions;


#[derive(InputObject)]
pub struct CreateLinkInput {
  pub url: String,
  pub label: String
}

#[derive(SimpleObject, Default, Serialize, Deserialize)]
pub struct Link {
  pub id: ID,
  pub url: String,
  pub label: String
}

impl From<LinkModel> for Link {
  fn from(model: LinkModel) -> Self {
    Link {
      id: ID::from(model.id.unwrap().to_hex()),
      url: model.url,
      label: model.label
    }
  }
}

#[Object]
impl LinkQueries {
  pub async fn get_all(&self, _ctx: &Context<'_>) -> Vec<Link> {
    let db = _ctx.data::<Database>().unwrap();

    let link_collection = db.collection::<LinkModel>("links");
    let mut cursor = link_collection.find(None, None).await.unwrap();
    let mut response: Vec<Link> = vec![];
    while let Some(model) = cursor.try_next().await.unwrap() {
      response.push(Link {
        id: ID::from(model.id.unwrap().to_hex()),
        label: model.label,
        url: model.url
      })
    };
    response
  }
}

#[Object]
impl LinkMutations {
  pub async fn insert_link(&self, _ctx: &Context<'_>, args: CreateLinkInput) -> Link {
    let db = _ctx.data::<Database>().unwrap();
    let mut pubsub = _ctx.data::<PubSub>().unwrap();

    let link = LinkModel::new(args.url, args.label);

    db.collection::<LinkModel>("links")
      .insert_one(&link, None).await.expect("Cannot insert");

    let msg = serde_json::to_string::<Link>(&Link::from(link.clone())).unwrap();
    let _ = pubsub.publish_client.publish::<String, _, String>("link_channel", msg).await;

    Link::from(link)
  }
}

#[Subscription]
impl LinkSubscriptions {
  pub async fn link_stream(&self, _ctx: &Context<'_>) -> impl Stream<Item = Link> {
    let pubsub = _ctx.data::<PubSub>().unwrap();
    pubsub.subscribe_client.subscribe("link_channel".to_string()).await.unwrap();
    let mut message_stream = pubsub.subscribe_client.on_message();
    stream! {
      while let Some((channel, message)) = message_stream.next().await {
        if let RedisValue::String(str) = message {
          let link = serde_json::from_str::<Link>(&str).unwrap();
          yield link;
        }
      }
    }
  }
}