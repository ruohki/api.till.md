use crate::graphql::GraphqlSchema;
use actix_web::guard::GuardContext;
use actix_web::http::header;
use actix_web::http::header::HeaderMap;
use actix_web::web::Query;
use actix_web::{get, post, web, HttpRequest, HttpResponse, Result};
use async_graphql::{
    http::{playground_source, GraphQLPlaygroundConfig},
    Data, Schema,
};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;

use crate::models::user::UserEntity;
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse, GraphQLSubscription};
use chrono::Utc;
use mongodb::bson::doc;
use mongodb::options::FindOneOptions;
use mongodb::Database;

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

async fn get_user_from_token(db: Arc<Database>, auth_token: String) -> Option<UserEntity> {
    let user_collection = db.collection::<UserEntity>("users");
    let options = FindOneOptions::builder()
        .projection(doc! { "password_hash": 0 })
        .build();
    if let Ok(Some(entity)) = user_collection.find_one(doc! { "access_token.token": auth_token, "access_token.expire": { "$gte": mongodb::bson::DateTime::from_millis(Utc::now().timestamp_millis()) } }, options).await {
        return Some(entity);
    }
    None
}

#[get("/graphql")]
pub async fn graphql_playground() -> Result<HttpResponse> {
    let playground = GraphQLPlaygroundConfig::new("/graphql").subscription_endpoint("/graphql");

    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(playground_source(playground)))
}

pub async fn on_connection_init(
    value: serde_json::Value,
    db: Arc<Database>
) -> async_graphql::Result<Data> {
    #[derive(Debug, Deserialize)]
    struct Payload {
        authorization: String,
    }
    let mut data = Data::default();

    if let Ok(payload) = serde_json::from_value::<Payload>(value) {
        if let Some(user) = get_user_from_token(db.clone(), payload.authorization).await {
            db.collection::<UserEntity>("users").update_one(doc! { "_id": user.id.unwrap() }, doc! { "$set": { "last_access": mongodb::bson::DateTime::from_millis(Utc::now().timestamp_millis() )}}, None).await.expect("Error updating timestamp");
            data.insert(user);
            return Ok(data);
        }
    }

    Ok(data)
}

pub async fn graphql_subscription(
    schema: web::Data<GraphqlSchema>,
    req: HttpRequest,
    payload: web::Payload,
    db: web::Data<Database>,
) -> Result<HttpResponse> {
    let db = db.into_inner().clone();

    GraphQLSubscription::new(Schema::clone(&*schema))
        .on_connection_init(|value| on_connection_init(value, db))
        .start(&req, payload)
}

fn get_auth_from_headers(headers: &HeaderMap) -> Option<String> {
    headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().map(|s| s.to_string()).ok())
}

#[post("/graphql")]
pub async fn graphql_request(
    schema: web::Data<GraphqlSchema>,
    db: web::Data<Database>,
    req: HttpRequest,
    gql_request: GraphQLRequest,
) -> GraphQLResponse {
    let mut request = gql_request.into_inner();

    if let Some(auth_token) = get_auth_from_headers(req.headers()) {
        //TODO: Make the user being cached in redis
        let db = db.into_inner();
        if let Some(entity) = get_user_from_token(db.clone(), auth_token).await {
            db.collection::<UserEntity>("users").update_one(doc! { "_id": entity.id.unwrap() }, doc! { "$set": { "last_access": mongodb::bson::DateTime::from_millis(Utc::now().timestamp_millis() )}}, None).await.expect("Error updating timestamp");
            request = request.data(entity);
        }
    }
    schema.execute(request).await.into()
}

#[get("/graphql", guard = "query_guard")]
pub async fn graphql_query(
    schema: web::Data<GraphqlSchema>,
    db: web::Data<Database>,
    req: HttpRequest,
    gql_request: GraphQLRequest,
) -> GraphQLResponse {
    let mut request = gql_request.into_inner();

    if let Some(query) = req.head().uri.query() {
        let query = Query::<HashMap<String, String>>::from_query(query).unwrap();
        if query.contains_key("authorization") {
            let auth_token = query.get("authorization").unwrap();
            let db = db.into_inner();
            if let Some(entity) = get_user_from_token(db.clone(), auth_token.clone()).await {
                db.collection::<UserEntity>("users").update_one(doc! { "_id": entity.id.unwrap() }, doc! { "$set": { "last_access": mongodb::bson::DateTime::from_millis(Utc::now().timestamp_millis() )}}, None).await.expect("Error updating timestamp");
                request = request.data(entity);
            }
        }
    }

    schema.execute(request).await.into()
}
