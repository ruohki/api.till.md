use async_graphql::{Context, Error, Object, Result};

use crate::graphql::user::inputs::{CreateAccessToken, CreateUserInput};
use crate::graphql::user::objects::{AccessToken, User};
use crate::models::user::{AccessTokenEntity, UserEntity};
use crate::password::verify_password;
use crate::ModelFor;
use chrono::{Duration, Utc};
use mongodb::bson::{doc, DateTime};
use uuid::Uuid;

pub mod inputs;
pub mod objects;

#[derive(Default)]
pub struct UserQueries;

#[derive(Default)]
pub struct UserMutations;

#[derive(Default)]
pub struct UserSubscriptions;

#[Object]
impl UserQueries {
  pub async fn get_user(&self, ctx: &Context<'_>, name: String) -> Result<User> {
    let users = ctx.data::<ModelFor<UserEntity>>().unwrap();
    if let Ok(Some(entity)) = users.find_one(doc! { "name": name }, None).await {
      return Ok(User::from(entity));
    }
    Err(Error::from("User not found."))
  }
}

#[Object]
impl UserMutations {
  pub async fn create_user(&self, ctx: &Context<'_>, user: CreateUserInput) -> Result<User> {
    let entity = UserEntity::new(user.name.clone(), user.email.clone(), user.password.clone());
    let users = ctx.data::<ModelFor<UserEntity>>().unwrap();
    if let Ok(val) = users
      .find_one(
        doc! { "$or": [{ "name": user.name }, { "email": user.email}] },
        None,
      )
      .await
    {
      if val.is_some() {
        return Err(Error::new("Username or email has already been taken"));
      }
    };

    match users.insert_one(&entity, None).await {
      Ok(_) => Ok(User::from(entity)),
      Err(_) => Err(Error::new("Cannot write to database")),
    }
  }

  pub async fn create_access_token(
    &self,
    ctx: &Context<'_>,
    args: CreateAccessToken,
  ) -> Result<AccessToken> {
    let filter = doc! {
          "$or": [{"name": args.name.clone() }, { "email_address": args.name.clone()}]
        };
    let users = ctx.data::<ModelFor<UserEntity>>().unwrap();
    if let Ok(result) = users
      .find_one(filter.clone(), None)
      .await
    {
      if let Some(user) = result {
        if let Ok(_) = verify_password(user.password_hash, args.password) {
          let token = base64::encode(Uuid::new_v4().to_string());
          let millis = Utc::now() + Duration::minutes(args.expire);
          let expire = match args.expire {
            0 => DateTime::parse_rfc3339_str("9999-12-31T23:59:59.00Z").unwrap(),
            _ => DateTime::from_millis(millis.timestamp_millis())
          };

          if let Ok(result) = users.update_one(filter, doc! { "$push": {  "access_token": { "token": token.clone(), "expire": expire.clone() }}}, None).await {
            let token = AccessToken::from(AccessTokenEntity::new(token, expire));
            return match result.modified_count {
              1 => {
                users.update_one(doc! { "_id": user.id.unwrap() }, doc! { "$set": { "last_login": mongodb::bson::DateTime::from_millis(Utc::now().timestamp_millis() )}}, None).await.expect("Error updating timestamp");
                Ok(token)
              }
              _ => Err(Error::new("Token could not be created"))
            };
          }
        }
      }
    };

    Err(Error::new(
      "A access token could not be created.".to_string(),
    ))
  }
}

/*#[Subscription]
impl ChannelSubscriptions {
  pub async fn listen_channel(&self, _ctx: &Context<'_>) -> impl Stream<Item=i32> {}
}
*/
