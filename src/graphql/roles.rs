use async_graphql::Enum;
use serde::{Deserialize, Serialize};

#[derive(Enum, Debug, Copy, Clone, Eq, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum Role {
  Root,
  Admin,
  User,
}

impl Role {
  pub fn as_str(&self) -> &'static str {
    match self {
      Role::Root => "Root",
      Role::Admin => "Admin",
      Role::User => "User"
    }
  }
}