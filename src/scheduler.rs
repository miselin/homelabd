use log::info;
use std::sync::Arc;
use tokio::time::{Duration, interval};

use crate::config::Config;

#[async_trait::async_trait]
pub trait Subsystem: Send + Sync {
    fn name(&self) -> &'static str;
    fn interval_seconds(&self) -> u64;
    async fn run(&self);
}

pub struct Scheduler {
    subsystems: Vec<Arc<dyn Subsystem>>,
}

impl Scheduler {
    pub fn new(_config: &Config) -> Self {
        Self {
            subsystems: Vec::new(),
        }
    }

    pub fn register<T: Subsystem + 'static>(&mut self, sub: T) {
        self.subsystems.push(Arc::new(sub));
    }

    pub async fn run(self) {
        let mut handles = Vec::new();
        for sub in self.subsystems {
            let task = sub.clone();
            handles.push(tokio::spawn(async move {
                info!(
                    "Starting subsystem: {} with interval {}",
                    task.name(),
                    task.interval_seconds()
                );
                let mut tick = interval(Duration::from_secs(task.interval_seconds()));
                loop {
                    tick.tick().await;
                    task.run().await;
                }
            }));
        }

        futures::future::join_all(handles).await;
    }
}
