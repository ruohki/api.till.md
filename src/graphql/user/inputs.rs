use async_graphql::*;

#[derive(InputObject)]
pub struct CreateUserInput {
  #[graphql(validator(min_length = 4, max_length = 64))]
  pub name: String,

  #[graphql(validator(email, max_length = 64))]
  pub email: String,

  #[graphql(validator(min_length = 4, max_length = 128))]
  pub password: String,
}

#[derive(InputObject)]
pub struct CreateAccessToken {
  #[graphql(validator(min_length = 4, max_length = 64))]
  pub name: String,

  #[graphql(validator(min_length = 4, max_length = 128))]
  pub password: String,

  // In minutes, 0 does not expire, 43200 = 1 Month
  #[graphql(default = 60, validator(minimum = 0, maximum = 43200))]
  pub expire: i64,
}