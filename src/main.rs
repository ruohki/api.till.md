#![feature(iterator_try_collect)]

mod graphql;
mod models;
mod password;
mod routes;
mod connections;

use crate::graphql::build_schema;
use crate::graphql::roles::Role;
use crate::models::model::ModelFor;

use actix_web::{guard, web, web::Data, App, HttpServer};
use routes::{gql::*, health::*};
use std::sync::{Arc, Mutex};
use sysinfo::{RefreshKind, SystemExt};
use std::env::var;
use lazy_static::lazy_static;
use crate::connections::{build_database_connection, build_pubsub_client};

lazy_static! {
    static ref MONGO_URL: String = var("MONGO_URL").expect("MONGO_URL not set in environment");
    static ref REDIS_URL: String = var("REDIS_URL").expect("REDIS_URL not set in environment");
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    let port = std::env::var("PORT").unwrap_or_else(|_| "8000".to_owned()).parse::<i32>().unwrap();
    let bind = std::env::var("BIND").unwrap_or_else(|_| "0.0.0.0".to_owned());


    // Initial system_info snapshot for state
    let sys = Systeminfo(Arc::new(Mutex::new(sysinfo::System::new_with_specifics(
        RefreshKind::new().with_cpu().with_memory(),
    ))));


    println!("{}", format!("Playground IDE: http://localhost:{}", port));

    let mongo_database = build_database_connection(&MONGO_URL).await.expect("Cannot connect to mongodb");
    let pubsub = build_pubsub_client(&REDIS_URL).await.expect("Cannot connect to redis");

    let schema = build_schema(mongo_database.clone(), pubsub).await;

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(schema.clone()))
            .app_data(Data::new(sys.clone()))
            .app_data(Data::new(mongo_database.clone()))
            // Get/Post to /graphql (Get guarded with custom guard to look for ?query=
            .service(graphql_request)
            .service(graphql_query)
            // Websocket subscription handler
            .service(
                web::resource("/graphql")
                    .guard(guard::Get())
                    .guard(guard::Header("upgrade", "websocket"))
                    .to(graphql_subscription),
            )
            // Playground endpoint
            .service(graphql_playground)
            .service(health)
    })
    .bind(format!("{}:{}", bind,port))?
    .run()
    .await
}
