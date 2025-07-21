use crate::config::Config;
use crate::net::send_multicast;
use crate::proto::homelabd::{Envelope, SystemInfoMessage};
use crate::scheduler::Subsystem;
use hostname::get;
use if_addrs::get_if_addrs;
use log::info;
use prost::Message;
use sys_info;

pub struct SystemInfo {
    interval: u64,
    version: String,
    config: Config,
}

impl SystemInfo {
    pub fn new(config: &Config, interval: u64) -> Self {
        Self {
            interval,
            version: env!("HOMELABD_VERSION").to_string(),
            config: config.clone(),
        }
    }
}

#[async_trait::async_trait]
impl Subsystem for SystemInfo {
    fn name(&self) -> &'static str {
        "SystemInfo"
    }

    fn interval_seconds(&self) -> u64 {
        self.interval
    }

    async fn run(&self) {
        let hostname = get().unwrap_or_default().to_string_lossy().into_owned();
        let uptime = match sys_info::boottime() {
            Ok(val) => val.tv_sec,
            Err(e) => {
                log::warn!(
                    "Failed to get system boot time, will set uptime to 0: {}",
                    e
                );
                0
            }
        };
        let addrs: Vec<String> = get_if_addrs()
            .unwrap_or_default()
            .iter()
            .filter(|ifa| ifa.ip().is_ipv4() && !ifa.ip().is_loopback())
            .map(|ifa| ifa.ip().to_string())
            .collect();

        let msg = Envelope {
            msg: Some(crate::proto::homelabd::envelope::Msg::SystemInfo(
                SystemInfoMessage {
                    hostname,
                    uptime,
                    ip: addrs,
                    homelabd_version: self.version.clone(),
                },
            )),
        };

        info!("Broadcasting system info: {:?}", msg);

        send_multicast(&self.config, msg.encode_to_vec().into())
            .await
            .unwrap_or_else(|e| {
                log::warn!("Failed to send system info: {}", e);
            });
    }
}
