use std::sync::{Arc, Mutex};
use rocket::serde::json::Json;
use rocket::State;
use serde::Serialize;
use sysinfo::{ProcessorExt, RefreshKind, System, SystemExt};

#[derive(Serialize)]
pub struct Health {
  pub uptime: u64,
  pub cpu_usage: Vec<f32>,
  pub memory: u64,
  pub memory_free: u64,
  pub memory_used: u64,
}

pub struct Systeminfo(pub Arc<Mutex<System>>);


#[get("/health")]
pub fn health(sysinfo_state: &State<Systeminfo>) -> Json<Health> {
  let mut sys = sysinfo_state.inner().0.lock().unwrap();
  sys.refresh_specifics(RefreshKind::new().with_memory().with_cpu());

  let cpu_usage = sys.processors().into_iter().map(|p| p.cpu_usage()).collect::<Vec<f32>>();

  Json(Health {
    memory:  sys.total_memory(),
    memory_used: sys.used_memory(),
    memory_free: sys.free_memory(),
    uptime: sys.uptime(),
    cpu_usage,
  })
}