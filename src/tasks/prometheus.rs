use crate::config::Config;
use crate::receivers::hostdb::HostDatabase;
use crate::scheduler::Schedulable;

use serde::Serialize;
use std::sync::Arc;

#[derive(Serialize)]
struct TargetGroup {
    targets: Vec<String>,
    labels: std::collections::HashMap<String, String>,
}

pub struct PrometheusEmitter {
    hostdb: Arc<HostDatabase>,
}

impl PrometheusEmitter {
    pub fn new(config: &Config, hostdb: Arc<HostDatabase>) -> Result<Self, String> {
        if !config.prometheus_discovery {
            return Err("Prometheus discovery is disabled in the configuration".to_string());
        }

        // Do we have /etc/prometheus to work with?
        if !std::path::Path::new("/etc/prometheus").exists() {
            return Err(
                "Prometheus configuration directory /etc/prometheus does not exist".to_string(),
            );
        }

        Ok(Self { hostdb })
    }
}

#[async_trait::async_trait]
impl Schedulable for PrometheusEmitter {
    fn name(&self) -> &'static str {
        "PrometheusEmitter"
    }

    fn interval_seconds(&self) -> u64 {
        60
    }

    async fn run(&self) {
        let hosts = self
            .hostdb
            .hosts()
            .iter()
            .map(|host| {
                let mut labels = std::collections::HashMap::new();
                labels.insert("job".to_string(), "homelabd".to_string());
                labels.insert("version".to_string(), host.version.clone());
                labels.insert("instance".to_string(), format!("{}:8800", host.name));

                TargetGroup {
                    targets: vec![format!("{}:8800", host.primaryip).to_string()],
                    labels,
                }
            })
            .collect::<Vec<_>>();

        let payload = serde_json::to_string_pretty(&hosts);
        if let Err(e) = payload {
            log::error!("Failed to serialize Prometheus targets: {}", e);
            return;
        }

        let file_path = "/etc/prometheus/homelabd.json";
        if let Err(e) = std::fs::write(file_path, payload.unwrap()) {
            log::error!("Failed to write Prometheus targets to {}: {}", file_path, e);
            return;
        }
        log::info!(
            "Prometheus targets written to {}: {}",
            file_path,
            serde_json::to_string_pretty(&hosts).unwrap()
        );
    }
}
