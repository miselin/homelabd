use log::info;
use std::sync::Arc;
use tokio::time::{Duration, interval};

use crate::config::Config;

#[async_trait::async_trait]
pub trait Schedulable: Send + Sync {
    fn name(&self) -> &'static str;
    fn interval_seconds(&self) -> u64;
    async fn run(&self);
}

pub struct Scheduler {
    schedulables: Vec<Arc<dyn Schedulable>>,
}

impl Scheduler {
    pub fn new(_config: &Config) -> Self {
        Self {
            schedulables: Vec::new(),
        }
    }

    pub fn register<T: Schedulable + 'static>(&mut self, sub: Arc<T>) {
        self.schedulables.push(sub);
    }

    pub async fn run(self) {
        let mut handles = Vec::new();
        for sub in self.schedulables {
            let task = Arc::clone(&sub);
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
