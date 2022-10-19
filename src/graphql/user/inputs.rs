use async_graphql::InputObject;

#[derive(InputObject)]
pub struct CreateUserInput {
  pub name: String,
  pub email: String,
  pub password: String,
}

#[derive(InputObject)]
pub struct CreateAccessToken {
  pub name: String,
  pub password: String
}