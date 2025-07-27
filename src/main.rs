mod config;
mod dispatch;
mod http;
mod metrics;
mod net;
mod proto;
mod receivers;
mod scheduler;
mod subsystems;
mod tasks;

use clap::Parser;
use config::Config;
use receivers::hostdb;
use receivers::prometheus::PrometheusEmitter;
use scheduler::Scheduler;
use std::sync::Arc;
use subsystems::{prometheus_scan, self_update, system_info};

#[tokio::main]
async fn main() {
    env_logger::init();
    let config = Arc::new(Config::parse());

    let mut scheduler = Scheduler::new(&config);
    let mut dispatcher = dispatch::Dispatcher::new();

    scheduler.register(Arc::new(system_info::SystemInfo::new(&config, 10)));
    scheduler.register(Arc::new(prometheus_scan::PrometheusScan::new(
        Arc::clone(&config),
        30,
    )));
    scheduler.register(Arc::new(self_update::SelfUpdateCheck::new(&config, 60)));

    let hostdb = Arc::new(hostdb::HostDatabase::new(&config));
    scheduler.register(Arc::clone(&hostdb));
    dispatcher.register(Arc::clone(&hostdb));

    if let Ok(prometheus_emitter) = PrometheusEmitter::new(&config, Arc::clone(&hostdb)) {
        let emitter = Arc::new(prometheus_emitter);
        dispatcher.register(Arc::clone(&emitter));
        scheduler.register(Arc::clone(&emitter));
    } else {
        log::warn!("Prometheus discovery is disabled or configuration is invalid.");
    }

    let multicast_config = Arc::clone(&config);
    let http_config = Arc::clone(&config);

    tokio::spawn(async move {
        if let Err(e) = net::start_multicast_listener(&multicast_config, dispatcher).await {
            log::error!("Failed to start multicast listener: {}", e);
            std::process::exit(1);
        }
    });
    tokio::spawn(async move {
        let http_server = Arc::new(http::HttpServer::new(http_config));
        if let Err(e) = http_server.start().await {
            log::error!("Failed to start HTTP server: {}", e);
            std::process::exit(1);
        }
    });

    scheduler.run().await;
}
