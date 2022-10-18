use crate::models::links::LinkModel;

use async_graphql::{Context, ID, Object, InputObject, SimpleObject};
use async_graphql::futures_util::TryStreamExt;
use mongodb::{Database};

#[derive(Default)]
pub struct LinkQueries;

#[derive(Default)]
pub struct LinkMutations;

#[derive(InputObject)]
pub struct CreateLinkInput {
  pub url: String,
  pub label: String
}

#[derive(SimpleObject, Default)]
pub struct Link {
  pub id: ID,
  pub url: String,
  pub label: String
}

#[Object]
impl LinkQueries {
  pub async fn get_all(&self, _ctx: &Context<'_>) -> Vec<Link> {
    let db = _ctx.data::<Database>().unwrap();

    let link_collection = db.collection::<LinkModel>("links").clone();
    let mut cursor = link_collection.find(None, None).await.unwrap();
    let mut response: Vec<Link> = vec![];
    while let Some(model) = cursor.try_next().await.unwrap() {
      response.push(Link {
        id: ID::from(model.id.unwrap().to_hex()),
        label: model.label,
        url: model.url
      })
    };
    response
  }
}

#[Object]
impl LinkMutations {
  pub async fn insert_link(&self, _ctx: &Context<'_>, args: CreateLinkInput) -> ID {
    let db = _ctx.data::<Database>().unwrap();

    let link_collection = db.collection::<LinkModel>("links").clone();
    let result = link_collection
      .insert_one(LinkModel::new(args.url, args.label), None).await.unwrap();

    ID::from(result.inserted_id.as_object_id().unwrap().to_hex())
  }
}