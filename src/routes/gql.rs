use crate::graphql::{GraphqlSchema};
use std::collections::HashMap;
use actix_web::{HttpRequest, HttpResponse, web, Result, get, post};
use actix_web::guard::{GuardContext};
use actix_web::http::header;
use actix_web::http::header::HeaderMap;
use actix_web::web::Query;
use async_graphql::{Schema, http::{GraphQLPlaygroundConfig, playground_source}};

use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse, GraphQLSubscription};
use chrono::Utc;
use mongodb::bson::doc;
use mongodb::Database;
use mongodb::options::FindOneOptions;
use crate::models::user::UserEntity;

#[derive(Debug)]
pub struct AuthToken(pub String);

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
  db: web::Data<Database>, req: HttpRequest, gql_request: GraphQLRequest
  payload: web::Payload,
) -> Result<HttpResponse> {
  let mut request = gql_request.into_inner();
  if let Some(auth_token) = get_auth_from_headers(req.headers()) {
    //TODO: Make the user beeing cached in redis
    let db = db.into_inner();
    let options = FindOneOptions::builder().projection(doc! { "password_hash": 0 }).build();
    let user_collection = db.collection::<UserEntity>("users");
    if let Ok(Some(entity)) = user_collection.find_one(doc! { "access_token.token": auth_token, "access_token.expire": { "$gte": mongodb::bson::DateTime::from_millis(Utc::now().timestamp_millis()) } }, options).await {
      user_collection.update_one(doc! { "_id": entity.id.unwrap() }, doc! { "$set": { "last_access": mongodb::bson::DateTime::from_millis(Utc::now().timestamp_millis() )}}, None).await.expect("Error updating timestamp");
      request = request.data(entity);
    }
  }
  GraphQLSubscription::new(Schema::clone(&*schema)).start(request, payload)
}

fn get_auth_from_headers(headers: &HeaderMap) -> Option<String> {
  headers
    .get(header::AUTHORIZATION)
    .and_then(|value| value.to_str().map(|s| s.to_string()).ok())
}

#[post("/graphql")]
pub async fn graphql_request(schema: web::Data<GraphqlSchema>, db: web::Data<Database>, req: HttpRequest, gql_request: GraphQLRequest) -> GraphQLResponse {
  let mut request = gql_request.into_inner();
  if let Some(auth_token) = get_auth_from_headers(req.headers()) {
    //TODO: Make the user beeing cached in redis
    let db = db.into_inner();
    let options = FindOneOptions::builder().projection(doc! { "password_hash": 0 }).build();
    let user_collection = db.collection::<UserEntity>("users");
    if let Ok(Some(entity)) = user_collection.find_one(doc! { "access_token.token": auth_token, "access_token.expire": { "$gte": mongodb::bson::DateTime::from_millis(Utc::now().timestamp_millis()) } }, options).await {
      user_collection.update_one(doc! { "_id": entity.id.unwrap() }, doc! { "$set": { "last_access": mongodb::bson::DateTime::from_millis(Utc::now().timestamp_millis() )}}, None).await.expect("Error updating timestamp");
      request = request.data(entity);
    }
  }
  schema.execute(request).await.into()
}


#[get("/graphql", guard = "query_guard")]
pub async fn graphql_query(schema: web::Data<GraphqlSchema>, db: web::Data<Database>, req: HttpRequest, gql_request: GraphQLRequest) -> GraphQLResponse {
  let mut request = gql_request.into_inner();

  if let Some(query) = req.head().uri.query() {
    let query = Query::<HashMap<String, String>>::from_query(query).unwrap();
    if query.contains_key("authorization") {
      let auth_token = query.get("authorization").unwrap();
      let db = db.into_inner();
      let options = FindOneOptions::builder().projection(doc! { "password_hash": 0 }).build();
      let user_collection = db.collection::<UserEntity>("users");
      if let Ok(Some(entity)) = user_collection.find_one(doc! { "access_token.token": auth_token, "access_token.expire": { "$gte": mongodb::bson::DateTime::from_millis(Utc::now().timestamp_millis()) } }, options).await {
        user_collection.update_one(doc! { "_id": entity.id.unwrap() }, doc! { "$set": { "last_access": mongodb::bson::DateTime::from_millis(Utc::now().timestamp_millis() )}}, None).await.expect("Error updating timestamp");
        request = request.data(entity);
      }
    }
  }

  schema.execute(request).await.into()
}