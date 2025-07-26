use crate::config::Config;
use crate::proto::homelabd::Envelope;
use crate::{dispatch::Dispatchable, scheduler::Schedulable};
use std::sync::{Arc, Mutex};
use std::time;

pub struct Host {
    name: String,
    ip: Vec<String>,
    uptime: i64,
    version: String,
}

struct HostEntry {
    host: Arc<Host>,
    last_seen: time::SystemTime,
}

struct Database {
    hosts: Vec<HostEntry>,
    host_lookup: std::collections::HashMap<String, usize>,
}

pub struct HostDatabase {
    db: Mutex<Database>,
}

// Maximum age for a host without a recent broadcast before it's considered stale and evicted
const MAX_HOST_AGE: time::Duration = time::Duration::from_secs(5 * 60);

impl Database {
    pub fn new() -> Self {
        Self {
            hosts: Vec::new(),
            host_lookup: std::collections::HashMap::new(),
        }
    }

    pub fn hosts(&self) -> &Vec<HostEntry> {
        &self.hosts
    }

    pub fn hosts_mut(&mut self) -> &mut Vec<HostEntry> {
        &mut self.hosts
    }

    pub fn host_lookup(&self) -> &std::collections::HashMap<String, usize> {
        &self.host_lookup
    }

    pub fn host_lookup_mut(&mut self) -> &mut std::collections::HashMap<String, usize> {
        &mut self.host_lookup
    }

    pub fn pair(&self) -> (&Vec<HostEntry>, &std::collections::HashMap<String, usize>) {
        (&self.hosts, &self.host_lookup)
    }

    pub fn pair_mut(
        &mut self,
    ) -> (
        &mut Vec<HostEntry>,
        &mut std::collections::HashMap<String, usize>,
    ) {
        (&mut self.hosts, &mut self.host_lookup)
    }
}

impl HostDatabase {
    pub fn new(_config: &Config) -> Self {
        Self {
            db: Mutex::new(Database::new()),
        }
    }

    pub fn host_seen(&self, host: Host) {
        let entry = HostEntry {
            host: Arc::new(host),
            last_seen: time::SystemTime::now(),
        };

        let mut db = self.db.lock().unwrap();

        let (hosts, hosts_lookup) = db.pair_mut();

        if let Some(index) = hosts_lookup.get(&entry.host.name) {
            log::info!("Updating host entry for {}", entry.host.name);
            hosts[*index] = entry;
        } else {
            let name = entry.host.name.clone();
            log::info!("Adding new host entry for {}", name);
            hosts.push(entry);
            hosts_lookup.insert(name, hosts.len() - 1);
        }
    }

    pub fn get_host(&self, name: &str) -> Option<Arc<Host>> {
        let db = self.db.lock().unwrap();

        let index = db.host_lookup().get(name)?;
        let entry = db.hosts().get(*index)?;

        Some(Arc::clone(&entry.host))
    }

    pub fn last_seen(&self, name: &str) -> Option<time::SystemTime> {
        let db = self.db.lock().unwrap();

        db.host_lookup()
            .get(name)
            .and_then(|&index| db.hosts().get(index))
            .map(|entry| entry.last_seen)
    }

    fn evict_old_hosts(&self, max_age: time::Duration) {
        let now = time::SystemTime::now();
        let mut db = self.db.lock().unwrap();
        log::info!("Evicting hosts older than {:?}", max_age);

        let (hosts, hosts_lookup) = db.pair_mut();

        let initial_hosts = hosts.len();

        hosts.retain(|entry| {
            now.duration_since(entry.last_seen)
                .unwrap_or(time::Duration::ZERO)
                < max_age
        });

        let num_evicted = initial_hosts - hosts_lookup.len();
        if num_evicted > 0 {
            log::info!("Evicted {} stale hosts", num_evicted);
        }

        // Rebuild the lookup table after evicting old hosts
        hosts_lookup.clear();
        for (index, entry) in hosts.iter().enumerate() {
            hosts_lookup.insert(entry.host.name.clone(), index);
        }
    }
}

#[async_trait::async_trait]
impl Schedulable for HostDatabase {
    fn name(&self) -> &'static str {
        "HostDatabase"
    }

    fn interval_seconds(&self) -> u64 {
        60
    }

    async fn run(&self) {
        self.evict_old_hosts(MAX_HOST_AGE);
    }
}

impl Dispatchable for HostDatabase {
    fn dispatcher_name(&self) -> &'static str {
        "HostDatabase"
    }

    fn dispatch(&self, msg: &Envelope) -> Result<(), String> {
        match &msg.msg {
            Some(crate::proto::homelabd::envelope::Msg::SystemInfo(sysinfo)) => {
                let host = Host {
                    name: sysinfo.hostname.clone(),
                    ip: sysinfo.ip.clone(),
                    uptime: sysinfo.uptime,
                    version: sysinfo.homelabd_version.clone(),
                };
                self.host_seen(host);
                Ok(())
            }
            _ => Err("Unsupported message type".to_string()),
        }
    }
}
