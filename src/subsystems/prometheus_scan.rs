use std::sync::Arc;

use crate::net::send_multicast;
use crate::proto::homelabd::{Envelope, PrometheusDiscoveryMessage, PrometheusExporter};
use crate::{config::Config, scheduler::Schedulable};
use hostname;
use log::info;
use procfs::process::all_processes;
use prost::Message;

pub struct PrometheusScan {
    interval: u64,
    registry: ExporterRegistry,
    config: Arc<Config>,
}

#[derive(Clone)]
struct ExporterCandidate {
    process_name: String,
    job: String,
    port: u16,
}

struct DiscoveredExporter {
    job: String,
    port: u16,
}

struct ExporterRegistry {
    candidates: Vec<ExporterCandidate>,
}

impl PrometheusScan {
    pub fn new(config: Arc<Config>, interval: u64) -> Self {
        Self {
            interval,
            registry: ExporterRegistry::new(),
            config,
        }
    }
}

#[async_trait::async_trait]
impl Schedulable for PrometheusScan {
    fn name(&self) -> &'static str {
        "PrometheusScan"
    }

    fn interval_seconds(&self) -> u64 {
        self.interval
    }

    async fn run(&self) {
        let hostname = hostname::get()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();

        // Search process list
        // - prometheus-node-exporter: job=node_exporter (port 9100)
        // - prometheus-bind-exporter: job=dns (port 9153)
        // - grafana: job=grafana (port 3000)
        // - prometheus: job=prometheus (port 9090)

        let exporters = self
            .registry
            .discover()
            .iter()
            .map(|exporter| PrometheusExporter {
                host: hostname.clone(),
                job: exporter.job.clone(),
                port: exporter.port as u32,
            })
            .collect::<Vec<PrometheusExporter>>();

        if exporters.is_empty() {
            info!("No Prometheus exporters found in the system.");
            return;
        }

        info!(
            "Discovered {} Prometheus exporters: {:?}",
            exporters.len(),
            exporters
        );

        let msg = Envelope {
            msg: Some(crate::proto::homelabd::envelope::Msg::PrometheusDiscovery(
                PrometheusDiscoveryMessage {
                    discovered_targets: exporters,
                },
            )),
        };

        send_multicast(&self.config, msg.encode_to_vec().into())
            .await
            .expect("Failed to send Prometheus discovery message");
    }
}

impl ExporterRegistry {
    fn new() -> Self {
        Self {
            candidates: vec![
                ExporterCandidate {
                    process_name: "prometheus-node-exporter".to_string(),
                    job: "node_exporter".to_string(),
                    port: 9100,
                },
                ExporterCandidate {
                    process_name: "prometheus-bind-exporter".to_string(),
                    job: "dns".to_string(),
                    port: 9153,
                },
                ExporterCandidate {
                    process_name: "prometheus-snmp-exporter".to_string(),
                    // Note: legacy job name from existing configuration
                    job: "snmp-exporter".to_string(),
                    port: 9116,
                },
                ExporterCandidate {
                    process_name: "grafana".to_string(),
                    job: "grafana".to_string(),
                    port: 3000,
                },
                ExporterCandidate {
                    process_name: "prometheus".to_string(),
                    job: "prometheus".to_string(),
                    port: 9090,
                },
                ExporterCandidate {
                    process_name: "etcd".to_string(),
                    job: "etcd".to_string(),
                    port: 2381,
                },
                ExporterCandidate {
                    process_name: "unpoller".to_string(),
                    job: "unifipoll".to_string(),
                    port: 9130,
                },
            ],
        }
    }

    fn candidate(&self, process_name: &str) -> Option<&ExporterCandidate> {
        self.candidates
            .iter()
            .find(|c| process_name.contains(&c.process_name))
    }

    fn discover(&self) -> Vec<DiscoveredExporter> {
        // get the full list of processes running on this system (Linux)
        let all_processes = all_processes();
        if all_processes.is_err() {
            log::warn!(
                "Failed to retrieve process list: {}",
                all_processes.err().unwrap()
            );
            return vec![];
        }

        all_processes
            .unwrap()
            .into_iter()
            .filter(|proc| proc.is_ok())
            .map(|proc| match proc {
                Ok(proc) => {
                    if let Ok(exe) = proc.exe() {
                        self.candidate(&exe.to_string_lossy())
                    } else {
                        None
                    }
                }
                Err(_) => None,
            })
            .filter(|c| c.is_some())
            .map(|c| c.unwrap())
            .map(|c| DiscoveredExporter {
                job: c.job.clone(),
                port: c.port as u16,
            })
            .collect::<Vec<_>>()
    }
}
