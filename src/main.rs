#[macro_use] extern crate rocket;

mod routes;

use std::sync::{Arc, Mutex};
use rocket::{Build, Rocket};
use sysinfo::{RefreshKind, SystemExt};
use routes::{health::*};

#[launch]
fn rocket() -> Rocket<Build> {
  // Initial system_info snapshot for state
  let sys = Systeminfo(
    Arc::new(Mutex::new(sysinfo::System::new_with_specifics(RefreshKind::new().with_cpu().with_memory())))
  );

  rocket::build()
    .mount("/", routes![health])
    .manage(sys)
}
