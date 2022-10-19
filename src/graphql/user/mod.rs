
use async_graphql::{Object, Context, Result, Error};

use chrono::{Duration, Utc};
use mongodb::bson::{DateTime, doc};
use mongodb::Database;
use uuid::Uuid;
use crate::graphql::user::inputs::{CreateAccessToken, CreateUserInput};
use crate::graphql::user::objects::{AccessToken, User};
use crate::models::user::{AccessTokenEntity, UserEntity};
use crate::password::verify_password;

pub mod inputs;
pub mod objects;

#[derive(Default)]
pub struct UserQueries;

#[derive(Default)]
pub struct UserMutations;

#[derive(Default)]
pub struct UserSubscriptions;

#[Object]
impl UserMutations {
  pub async fn create_user(&self, _ctx: &Context<'_>, user: CreateUserInput) -> Result<User> {
    let entity = UserEntity::new(user.name.clone(), user.email.clone(), user.password.clone());

    let db = _ctx.data::<Database>().unwrap();

    let users = db.collection::<UserEntity>("users");
    if let Ok(val) = users.find_one(doc! { "$or": [{ "name": user.name }, { "email": user.email}] }, None).await {
      if val.is_some() {
        return Err(Error::new("Username or email has already been taken"))
      }
    };

    match users.insert_one(&entity, None).await {
      Ok(_) => Ok(User::from(entity)),
      Err(_) => Err(Error::new("Cannot write to database"))
    }
  }

  pub async fn create_access_token(&self, _ctx: &Context<'_>, args: CreateAccessToken) -> Result<AccessToken> {
    let filter = doc! {
      "$or": [{"name": args.name.clone() }, { "email_address": args.name.clone()}]
    };
    let db = _ctx.data::<Database>().unwrap();
    if let Ok(result) = db.collection::<UserEntity>("users").find_one(filter.clone(), None).await {
      if let Some(user) = result {
        if let Ok(_) = verify_password(user.password_hash, args.password) {
          let token = base64::encode(Uuid::new_v4().to_string());
          let millis = Utc::now() + Duration::hours(1);
          let expire = DateTime::from_millis(millis.timestamp_millis());

          if let Ok(result) = db.collection::<UserEntity>("users").update_one(filter, doc! { "$push": {  "access_token": { "token": token.clone(), "expire": expire.clone() }}}, None).await {
            let token = AccessToken::from(AccessTokenEntity::new(token, expire));
            return match result.modified_count {
              1 => {
                db.collection::<UserEntity>("users").update_one(doc! { "_id": user.id.unwrap() }, doc! { "$set": { "last_login": mongodb::bson::DateTime::from_millis(Utc::now().timestamp_millis() )}}, None).await.expect("Error updating timestamp");
                Ok(token)
              },
              _ => Err(Error::new("Token could not be created"))
            }
          }
        }
      }
    };

    Err(Error::new("A access token could not be created.".to_string()))
  }
}

/*#[Subscription]
impl ChannelSubscriptions {
  pub async fn listen_channel(&self, _ctx: &Context<'_>) -> impl Stream<Item=i32> {}
}
*/