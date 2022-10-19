use async_graphql::{Object, Context, Result, Error, Guard};
use mongodb::Database;
use crate::graphql::channel::inputs::CreateChannelInput;
use crate::graphql::channel::objects::Channel;
use crate::models::channel::ChannelEntity;

pub mod inputs;
pub mod objects;

#[derive(Default)]
pub struct ChannelQueries;

#[derive(Default)]
pub struct ChannelMutations;

#[derive(Default)]
pub struct ChannelSubscriptions;

#[derive(Eq, PartialEq, Copy, Clone)]
enum Role {
  Admin,
  Guest,
}

struct RoleGuard {
  role: Role,
}

impl RoleGuard {
  fn new(role: Role) -> Self {
    Self { role }
  }
}

#[async_trait::async_trait]
impl Guard for RoleGuard {
  async fn check(&self, ctx: &Context<'_>) -> Result<()> {
    if ctx.data_opt::<Role>() == Some(&self.role) {
      Ok(())
    } else {
      Err("Forbidden".into())
    }
  }
}

#[Object]
impl ChannelMutations {
  #[graphql(guard = "RoleGuard::new(Role::Admin)")]
  pub async fn create_channel(&self, _ctx: &Context<'_>, channel: CreateChannelInput) -> Result<Channel> {
    let entity = ChannelEntity::new(channel.name, channel.description, channel.public);

    let db = _ctx.data::<Database>().unwrap();

    match db.collection::<ChannelEntity>("channel")
      .insert_one(&entity, None).await {
      Ok(_) => Ok(Channel::from(entity)),
      Err(_) => Err(Error::new("Cannot write to database"))
    }
  }
}

/*#[Subscription]
impl ChannelSubscriptions {
  pub async fn listen_channel(&self, _ctx: &Context<'_>) -> impl Stream<Item=i32> {}
}
*/