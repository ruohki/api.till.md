#![feature(iterator_try_collect)]

mod routes;
mod graphql;
mod models;

use actix_web::{guard, web, web::Data, App, HttpServer};
use std::sync::{Arc, Mutex};
use sysinfo::{RefreshKind, SystemExt};
use crate::graphql::build_schema;
use routes::{health::*, gql::*};


#[actix_web::main]
async fn main() -> std::io::Result<()> {


  // Initial system_info snapshot for state
  let sys = Systeminfo(
    Arc::new(Mutex::new(sysinfo::System::new_with_specifics(RefreshKind::new().with_cpu().with_memory())))
  );

  let schema = build_schema().await;

  println!("Playground IDE: http://localhost:8000");

  HttpServer::new(move || {
    App::new()
      .app_data(Data::new(schema.clone()))
      .app_data(Data::new(sys.clone()))
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