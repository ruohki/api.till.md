use crate::password::hash_password;
use crate::Role;
use chrono::Utc;
use mongodb::bson::oid::ObjectId;
use mongodb::bson::DateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessTokenEntity {
    pub token: String,
    pub expire: DateTime,
}

impl AccessTokenEntity {
    pub fn new(t: String, e: DateTime) -> Self {
        Self {
            token: t,
            expire: e,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserEntity {
    #[serde(rename = "_id")]
    pub id: Option<ObjectId>,

    pub name: String,
    pub email_address: String,
    pub email_verified: bool,

    pub roles: Vec<Role>,

    #[serde(default)]
    pub password_hash: String,

    pub access_token: Vec<AccessTokenEntity>,
    pub when_created: DateTime,
    pub last_login: DateTime,
    pub last_access: DateTime,
}

impl UserEntity {
    pub fn new(name: String, email: String, password: String) -> Self {
        let password_hash = hash_password(password).unwrap();

        Self {
            id: Some(ObjectId::new()),
            name,
            email_address: email,
            email_verified: true,
            password_hash,
            roles: vec![],
            access_token: vec![],
            when_created: DateTime::from_millis(Utc::now().timestamp_millis()),
            last_login: DateTime::from_millis(Utc::now().timestamp_millis()),
            last_access: DateTime::from_millis(Utc::now().timestamp_millis()),
        }
    }
}
