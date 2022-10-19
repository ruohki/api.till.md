use async_graphql::InputObject;

#[derive(InputObject)]
pub struct AddRoleInput {
  pub name_or_id: String,
  pub role: String
}