use crate::{config::Config, scheduler::Subsystem};
use log::info;

pub struct SelfUpdateCheck {
    interval: u64,
}

impl SelfUpdateCheck {
    pub fn new(_config: &Config, interval: u64) -> Self {
        Self { interval }
    }
}

#[async_trait::async_trait]
impl Subsystem for SelfUpdateCheck {
    fn name(&self) -> &'static str {
        "SelfUpdateCheck"
    }

    fn interval_seconds(&self) -> u64 {
        self.interval
    }

    async fn run(&self) {
        // TODO: Check versions from peers and trigger update if needed
        info!("SelfUpdateCheck: (simulated) checking for new versions...");
    }
}
