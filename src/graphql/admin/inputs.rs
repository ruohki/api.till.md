use async_graphql::InputObject;
use crate::graphql::roles::Role;

#[derive(InputObject)]
pub struct AddRoleInput {
  pub name_or_id: String,
  pub role: Role
}

#[derive(InputObject)]
pub struct RemoveRoleInput {
  pub name_or_id: String,
  pub role: Role
}