use async_graphql::*;

#[derive(InputObject)]
pub struct CreateChannelInput {
  #[graphql(validator(min_length = 4, max_length = 64))]
  pub name: String,

  #[graphql(validator(min_length = 0, max_length = 1024))]
  pub description: String,
  pub public: bool,
}