use crate::{config::Config, scheduler::Schedulable};
use log::info;

pub struct PrometheusScan {
    interval: u64,
}

impl PrometheusScan {
    pub fn new(_config: &Config, interval: u64) -> Self {
        Self { interval }
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
        // TODO: Implement port/process scan logic
        info!("PrometheusScan: (simulated) scanning local processes for exporters...");
    }
}
