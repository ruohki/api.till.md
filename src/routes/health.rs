use actix_web::{get, web, Responder, Result};
use serde::Serialize;
use std::sync::{Arc, Mutex};
use sysinfo::{ProcessorExt, RefreshKind, System, SystemExt};

#[derive(Serialize)]
pub struct Health {
    pub uptime: u64,
    pub cpu_usage: Vec<f32>,
    pub memory: u64,
    pub memory_free: u64,
    pub memory_used: u64,
}

#[derive(Clone)]
pub struct Systeminfo(pub Arc<Mutex<System>>);

#[get("/health")]
pub async fn health(sysinfo_state: web::Data<Systeminfo>) -> Result<impl Responder> {
    let mut sys = sysinfo_state.0.lock().unwrap();

    sys.refresh_specifics(RefreshKind::new().with_memory().with_cpu());

    let cpu_usage = sys
        .processors()
        .into_iter()
        .map(|p| p.cpu_usage())
        .collect::<Vec<f32>>();

    Ok(web::Json(Health {
        memory: sys.total_memory(),
        memory_used: sys.used_memory(),
        memory_free: sys.free_memory(),
        uptime: sys.uptime(),
        cpu_usage,
    }))
}
