use async_graphql::async_stream::stream;
use async_graphql::futures_util::Stream;
use async_graphql::{Context, Error, Object, Result, Subscription, ID};
use chrono::Utc;
use fred::interfaces::PubsubInterface;
use fred::prelude::RedisValue;
use std::str::FromStr;

use crate::graphql::channel::inputs::{CreateChannelInput, SendChannelMessageInput};
use crate::graphql::channel::objects::{Channel, ChannelMessage};
use crate::graphql::guards::{AuthGuard, RoleGuard};
use crate::graphql::user::objects::User;
use crate::graphql::{roles, PubSub};
use crate::models::channel::ChannelEntity;
use crate::models::user::UserEntity;
use futures::stream::{StreamExt, TryStreamExt};
use mongodb::bson::{doc, oid::ObjectId};
use mongodb::Database;
use roles::Role;
use crate::ModelFor;

pub mod inputs;
pub mod objects;

#[derive(Default)]
pub struct ChannelQueries;

#[derive(Default)]
pub struct ChannelMutations;

#[derive(Default)]
pub struct ChannelSubscriptions;

#[Object]
impl ChannelQueries {
  #[graphql(guard = "AuthGuard")]
  pub async fn list_channel(&self, ctx: &Context<'_>) -> Result<Vec<Channel>> {
    let db = ctx.data::<Database>().unwrap();
    let channel_collection = db.collection::<ChannelEntity>("channel");

    let filter = doc! { "public": true };

    if let Ok(cursor) = channel_collection.find(filter, None).await {
      let documents: Vec<_> = cursor.try_collect().await?;
      return Ok(documents
        .into_iter()
        .map(|c| Channel::from(c))
        .collect::<Vec<Channel>>());
    }

    Ok(vec![])
  }
}

#[Object]
impl ChannelMutations {
  #[graphql(guard = "AuthGuard")]
  pub async fn create_channel(
    &self,
    ctx: &Context<'_>,
    channel: CreateChannelInput,
  ) -> Result<Channel> {
    let entity = ChannelEntity::new(channel.name, channel.description, channel.public);

    let db = ctx.data::<Database>().unwrap();

    match db
      .collection::<ChannelEntity>("channel")
      .insert_one(&entity, None)
      .await
    {
      Ok(_) => Ok(Channel::from(entity)),
      Err(_) => Err(Error::new("Cannot write to database")),
    }
  }

  #[graphql(guard = "RoleGuard::new(Role::Admin)")]
  pub async fn remove_channel(&self, _ctx: &Context<'_>, channel: String) -> Result<bool> {
    let db = _ctx.data::<Database>().unwrap();

    match db
      .collection::<ChannelEntity>("channel")
      .delete_one(
        doc! { "_id": ObjectId::from_str(channel.as_str()).unwrap() },
        None,
      )
      .await
    {
      Ok(_) => Ok(true),
      Err(_) => Err(Error::new("Cannot write to database")),
    }
  }

  #[graphql(guard = "AuthGuard")]
  pub async fn send_message_to_channel(
    &self,
    ctx: &Context<'_>,
    message: SendChannelMessageInput,
  ) -> Result<ChannelMessage> {
    let user = ctx.data::<UserEntity>().unwrap();
    let pubsub = ctx.data::<PubSub>().unwrap();
    let channels = ctx.data::<ModelFor<ChannelEntity>>().unwrap();

    let id = ObjectId::from_str(message.channel.as_str()).map_err(|_| Error::new("Invalid channel ID"))?;

    match channels.find_one(
      doc! { "_id": id },
      None,
    ).await {
      Ok(Some(channel)) => {
        let message = ChannelMessage {
          id: ID::from(ObjectId::new().to_hex()),
          message: message.message,
          send_to: Channel::from(channel.clone()),
          send_from: User::from(user.clone()),
          send_when: Utc::now().timestamp_millis(),
        };

        let msg = serde_json::to_string::<ChannelMessage>(&message).unwrap();
        let _ = pubsub
          .publish_client
          .publish::<String, _, String>(channel.id.unwrap().to_hex(), msg)
          .await;
        Ok(message)
      }
      Err(_) => Err(Error::new("Invalid channel ID")),
      _ => Err(Error::new("Unknown channel ID"))
    }
  }
}

#[Subscription]
impl ChannelSubscriptions {
  #[graphql(guard = "AuthGuard")]
  pub async fn listen_channel(
    &self,
    ctx: &Context<'_>,
    channel: ID,
  ) -> impl Stream<Item=ChannelMessage> {
    let pubsub = ctx.data::<PubSub>().unwrap();
    pubsub
      .subscribe_client
      .subscribe(channel.as_str())
      .await
      .unwrap();
    let mut message_stream = pubsub.subscribe_client.on_message();
    stream! {
      while let Some((_channel, message)) = message_stream.next().await {
        if let RedisValue::String(str) = message {
          let message = serde_json::from_str::<ChannelMessage>(&str).unwrap();
          yield message;
        }
      }
    }
  }
}
