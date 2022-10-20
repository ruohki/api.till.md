use crate::graphql::FromOid;
use crate::models::user::{AccessTokenEntity, UserEntity};
use async_graphql::{SimpleObject, ID};
use serde::{Deserialize, Serialize};

#[derive(SimpleObject)]
pub struct AccessToken {
    pub token: String,
    pub expire: i64,
}

impl From<AccessTokenEntity> for AccessToken {
    fn from(t: AccessTokenEntity) -> Self {
        Self {
            token: t.token,
            expire: t.expire.timestamp_millis(),
        }
    }
}

#[derive(SimpleObject, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: ID,
    pub name: String,
    pub email_address: String,
    pub email_verified: bool,
    pub roles: Vec<String>,
    pub when_created: i64,
    pub last_login: i64,
}

impl From<UserEntity> for User {
    fn from(e: UserEntity) -> Self {
        User {
            id: ID::from_object_id(e.id.unwrap()),
            name: e.name,
            email_address: e.email_address,
            email_verified: e.email_verified,
            roles: e
                .roles
                .into_iter()
                .map(|r| String::from(r.as_str()))
                .collect::<Vec<String>>(),
            last_login: e.last_login.timestamp_millis(),
            when_created: e.when_created.timestamp_millis(),
        }
    }
}
