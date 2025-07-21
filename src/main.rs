mod config;
mod dispatch;
mod http;
mod metrics;
mod net;
mod proto;
mod scheduler;
mod subsystems;

use clap::Parser;
use config::Config;
use scheduler::Scheduler;
use subsystems::{prometheus_scan, self_update, system_info};

#[tokio::main]
async fn main() {
    env_logger::init();
    let config = Config::parse();

    let mut scheduler = Scheduler::new(&config);
    scheduler.register(system_info::SystemInfo::new(&config, 10));
    scheduler.register(prometheus_scan::PrometheusScan::new(&config, 30));
    scheduler.register(self_update::SelfUpdateCheck::new(&config, 60));

    let multicast_config = config.clone();
    let http_config = config.clone();

    tokio::spawn(async move {
        let dispatcher = dispatch::Dispatcher::new();
        if let Err(e) = net::start_multicast_listener(&multicast_config, dispatcher).await {
            log::error!("Failed to start multicast listener: {}", e);
            std::process::exit(1);
        }
    });
    tokio::spawn(async move {
        if let Err(e) = http::start_http_server(&http_config).await {
            log::error!("Failed to start HTTP server: {}", e);
            std::process::exit(1);
        }
    });

    scheduler.run().await;
}
