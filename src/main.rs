#![feature(iterator_try_collect)]

mod graphql;
mod models;
mod password;
mod routes;

use crate::graphql::build_schema;
use crate::graphql::roles::Role;
use crate::models::model::ModelFor;

use actix_web::{guard, web, web::Data, App, HttpServer};
use mongodb::options::{ClientOptions, ResolverConfig};
use mongodb::Client;
use routes::{gql::*, health::*};
use std::sync::{Arc, Mutex};
use sysinfo::{RefreshKind, SystemExt};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let options = ClientOptions::parse_with_resolver_config(
        "mongodb://localhost:27017/rust",
        ResolverConfig::cloudflare(),
    )
    .await
    .unwrap();
    let mongo_client = Client::with_options(options).unwrap();
    let mongo_database = mongo_client.default_database().unwrap();

    // Initial system_info snapshot for state
    let sys = Systeminfo(Arc::new(Mutex::new(sysinfo::System::new_with_specifics(
        RefreshKind::new().with_cpu().with_memory(),
    ))));

    /*  let user_model = ModelFor::<UserEntity>::new(Arc::new(mongo_database), "users".to_owned());

    */
    let schema = build_schema().await;

    println!("Playground IDE: http://localhost:8000");

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
    .bind("0.0.0.0:8000")?
    .run()
    .await
}
