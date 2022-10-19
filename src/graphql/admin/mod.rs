use async_graphql::{Object, Context, Result, Error, Guard};
use mongodb::bson::doc;
use mongodb::Database;
use mongodb::options::{FindOneAndUpdateOptions, ReturnDocument};
use serde::de::Unexpected::Option;
use crate::graphql::admin::inputs::AddRoleInput;
use crate::models::user::UserEntity;

pub mod inputs;
pub mod objects;

#[derive(Default)]
pub struct AdminQueries;

#[derive(Default)]
pub struct AdminMutations;

#[derive(Default)]
pub struct AdminSubscriptions;

#[Object]
impl AdminMutations {
  pub async fn add_role(&self, _ctx: &Context<'_>, args: AddRoleInput) -> Result<bool> {
    let db = _ctx.data::<Database>().unwrap();
    let filter = doc! { "$or": [{ "_id": args.name_or_id.clone() }, { "name": args.name_or_id.clone() }] };
    let update = doc! { "$addToSet": { "roles": args.role.clone() }};

    let options = FindOneAndUpdateOptions::builder().return_document(ReturnDocument::After).build();

    if let Ok(response) = db.collection::<UserEntity>("users").find_one_and_update(filter, update, options).await {
      println!("{:?}", response);
      if let Some(entity) = response {
        return Ok(true);
      }
    }
    Err(Error::new(format!("Cannot grant role '{}' to user '{}'.", args.role, args.name_or_id )))
  }
}

/*#[Subscription]
impl ChannelSubscriptions {
  pub async fn listen_channel(&self, _ctx: &Context<'_>) -> impl Stream<Item=i32> {}
}
*/