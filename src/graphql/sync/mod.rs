use async_graphql::async_stream::stream;
use async_graphql::futures_util::Stream;
use async_graphql::{Context, Error, Object, Result, Subscription, ID};
use chrono::Utc;
use fred::interfaces::PubsubInterface;
use fred::prelude::RedisValue;
use std::str::FromStr;
use async_graphql::parser::types::OperationType;

use crate::graphql::channel::inputs::{CreateChannelInput, SendChannelMessageInput};
use crate::graphql::channel::objects::{Channel, ChannelMessage};
use crate::graphql::guards::{AuthGuard, RoleGuard};
use crate::graphql::user::objects::User;
use crate::graphql::{roles, PubSub};
use crate::models::channel::ChannelEntity;
use crate::models::user::UserEntity;
use futures::stream::{StreamExt, TryStreamExt};
use mongodb::bson::{doc, oid::ObjectId};
use roles::Role;
use crate::graphql::sync::inputs::{CreateArgs, ObjectType};
use crate::graphql::sync::inputs::ObjectType::File;
use crate::graphql::sync::objects::{CreateMessage, FileOperation, Operation, PathOperation, Stat, SyncEvent};
use crate::graphql::sync::objects::SyncEvent::Create;
use crate::ModelFor;

pub mod inputs;
pub mod objects;

#[derive(Default)]
pub struct SyncQueries;

#[derive(Default)]
pub struct SyncMutations;

#[derive(Default)]
pub struct SyncSubscriptions;

#[Object]
impl SyncMutations {
  #[graphql(guard = "AuthGuard")]
  pub async fn create_file_or_folder(&self, ctx: &Context<'_>, vault_id: String, args: CreateArgs) -> Result<SyncEvent> {
    let user = ctx.data::<UserEntity>().unwrap();
    let pubsub = ctx.data::<PubSub>().unwrap();

    let message = Create(CreateMessage {
      operation_type: args.object_type.clone(),
      operation: match args.object_type.clone()  {
        File => Operation::File(FileOperation {
          name: args.name.clone(),
          extension: args.extension.clone().unwrap(),
          path: args.path.clone(),
          basename: format!("{}.{}", args.name.clone(), args.extension.clone().unwrap()),
          stat: match args.stat {
            Some(stat) => Some(Stat::from_args(stat)),
            None => None
          }
        }),
        ObjectType::Folder => Operation::Path(PathOperation {
          name: args.name.clone(),
          path: args.path.clone(),
          basename: args.name,
          stat: match args.stat {
            Some(stat) => Some(Stat::from_args(stat)),
            None => None
          }
        })
      }
    });

    let msg = serde_json::to_string::<SyncEvent>(&message).unwrap();
    let _ = pubsub
      .publish
      .publish::<String, _, String>(vault_id.as_str(), msg)
      .await;
    Ok(message)

  }
}

/*#[Object]
impl SyncQueries {
  #[graphql(guard = "AuthGuard")]
  pub async fn list_channel(&self, ctx: &Context<'_>) -> Result<Vec<Channel>> {
    let channel_collection = ctx.data::<ModelFor<ChannelEntity>>().unwrap();

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
impl SyncMutations {
  #[graphql(guard = "AuthGuard")]
  pub async fn rename_file(
    &self,
    ctx: &Context<'_>,
  ) {

  }

  #[graphql(guard = "AuthGuard")]
  pub async fn create_channel(
    &self,
    ctx: &Context<'_>,
    channel: CreateChannelInput,
  ) -> Result<Channel> {
    let channel_collection = ctx.data::<ModelFor<ChannelEntity>>().unwrap();
    let entity = ChannelEntity::new(channel.name, channel.description, channel.public);


    match channel_collection
      .insert_one(&entity, None)
      .await
    {
      Ok(_) => Ok(Channel::from(entity)),
      Err(_) => Err(Error::new("Cannot write to database")),
    }
  }

  #[graphql(guard = "RoleGuard::new(Role::Admin)")]
  pub async fn remove_channel(&self, ctx: &Context<'_>, channel: String) -> Result<bool> {
    let channel_collection = ctx.data::<ModelFor<ChannelEntity>>().unwrap();

    match channel_collection
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
    args: SendChannelMessageInput,
  ) -> Result<ChannelMessage> {
    let user = ctx.data::<UserEntity>().unwrap();
    let pubsub = ctx.data::<PubSub>().unwrap();
    let channels = ctx.data::<ModelFor<ChannelEntity>>().unwrap();

    let id = ObjectId::from_str(args.channel.as_str()).map_err(|_| Error::new("Invalid channel ID"))?;

    match channels.find_one(
      doc! { "_id": id },
      None,
    ).await {
      Ok(Some(channel)) => {
        let message = ChannelMessage {
          id: ID::from(ObjectId::new().to_hex()),
          message: args.message,
          send_to: Channel::from(channel.clone()),
          send_from: User::from(user.clone()),
          send_when: Utc::now().timestamp_millis(),
        };

        let msg = serde_json::to_string::<ChannelMessage>(&message).unwrap();
        let _ = pubsub
          .publish
          .publish::<String, _, String>(args.channel.as_str(), msg)
          .await;
        Ok(message)
      }
      Err(_) => Err(Error::new("Invalid channel ID")),
      _ => Err(Error::new("Unknown channel ID"))
    }
  }
}*/

#[Subscription]
impl SyncSubscriptions {
  #[graphql(guard = "AuthGuard")]
  pub async fn listen_sync_events(
    &self,
    ctx: &Context<'_>,
    vault_id: ID,
  ) -> impl Stream<Item=SyncEvent> {

    let pubsub = ctx.data::<PubSub>().unwrap();
    pubsub
      .subscribe
      .subscribe(vault_id.as_str())
      .await
      .expect("Error subscribing to vault events");
    let mut message_stream = pubsub.subscribe.on_message();
    stream! {
      while let Some((_channel, message)) = message_stream.next().await {
        if let RedisValue::String(str) = message {
          let event = serde_json::from_str::<SyncEvent>(&str).unwrap();
          yield event;
        }
      }
    }
  }
}
