use crate::graphql::{GraphqlSchema};
use std::collections::HashMap;
use actix_web::{HttpRequest, HttpResponse, web, Result, get, post};
use actix_web::guard::{GuardContext};
use actix_web::web::Query;
use async_graphql::{Schema, http::{GraphQLPlaygroundConfig, playground_source}};

use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse, GraphQLSubscription};


// Guard to separate /graphql requests from /graphql?query= requests
fn query_guard(req: &GuardContext) -> bool {
  if let Some(query) = req.head().uri.query() {
    let query = Query::<HashMap<String, String>>::from_query(query).unwrap();
    return query.contains_key("query");
  }
  false
}

#[get("/graphql")]
pub async fn graphql_playground() -> Result<HttpResponse> {
  let playground = GraphQLPlaygroundConfig::new("/graphql")
    .subscription_endpoint("/graphql");

  Ok(HttpResponse::Ok()
    .content_type("text/html; charset=utf-8")
    .body(playground_source(playground)))
}


pub async fn graphql_subscription(
  schema: web::Data<GraphqlSchema>,
  req: HttpRequest,
  payload: web::Payload,
) -> Result<HttpResponse> {
  GraphQLSubscription::new(Schema::clone(&*schema)).start(&req, payload)
}

#[post("/graphql")]
pub async fn graphql_request(schema: web::Data<GraphqlSchema>, req: GraphQLRequest) -> GraphQLResponse {
  schema.execute(req.into_inner()).await.into()
}

#[get("/graphql", guard = "query_guard")]
pub async fn graphql_query(schema: web::Data<GraphqlSchema>, req: GraphQLRequest) -> GraphQLResponse {
  schema.execute(req.into_inner()).await.into()
}