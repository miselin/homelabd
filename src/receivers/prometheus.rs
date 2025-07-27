use crate::config::Config;
use crate::dispatch::Dispatchable;
use crate::proto::homelabd::{Envelope, PrometheusDiscoveryMessage, PrometheusExporter};
use crate::receivers::hostdb::HostDatabase;
use crate::scheduler::Schedulable;

use serde::Serialize;
use std::sync::{Arc, Mutex};

#[derive(Serialize)]
struct TargetGroup {
    targets: Vec<String>,
    labels: std::collections::HashMap<String, String>,
}

pub struct PrometheusEmitter {
    hostdb: Arc<HostDatabase>,
    discovered_targets: Mutex<Vec<PrometheusExporter>>,
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

        Ok(Self {
            hostdb,
            discovered_targets: Mutex::new(Vec::new()),
        })
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
        // Load HostDB targets for monitoring homelabd instances
        let mut targets = self
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

        // Add discovered Prometheus exporters
        let my_targets = self.discovered_targets.lock().unwrap();
        for exporter in my_targets.iter() {
            let mut labels = std::collections::HashMap::new();
            labels.insert("job".to_string(), exporter.job.clone());
            labels.insert(
                "instance".to_string(),
                format!("{}:{}", exporter.host, exporter.port),
            );

            // Get host from HostDB for its primary IP
            if let Some(host) = self.hostdb.get_host(&exporter.host) {
                targets.push(TargetGroup {
                    targets: vec![format!("{}:{}", host.primaryip, exporter.port)],
                    labels,
                });
            } else {
                log::warn!("No host found in HostDB for exporter: {}", exporter.host);
            }
        }

        let payload = serde_json::to_string_pretty(&targets);
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
            serde_json::to_string_pretty(&targets).unwrap()
        );
    }
}

impl Dispatchable for PrometheusEmitter {
    fn dispatcher_name(&self) -> &'static str {
        "PrometheusEmitter"
    }

    fn dispatch(&self, msg: &Envelope) -> Result<(), String> {
        match &msg.msg {
            Some(crate::proto::homelabd::envelope::Msg::PrometheusDiscovery(discovery)) => {
                let mut my_targets = self.discovered_targets.lock().unwrap();
                my_targets.clear();
                my_targets.extend(discovery.discovered_targets.iter().cloned());
                log::info!(
                    "Discovered {} Prometheus exporters: {:?}",
                    my_targets.len(),
                    my_targets
                );
                Ok(())
            }
            _ => Ok(()),
        }
    }
}
