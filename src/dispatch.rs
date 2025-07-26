use crate::metrics;
use crate::proto::homelabd::Envelope;

use prost::Message;
use std::sync::Arc;

pub trait Dispatchable: Send + Sync {
    fn dispatcher_name(&self) -> &'static str;

    fn dispatch(&self, message: &Envelope) -> Result<(), String>;
}

pub struct Dispatcher {
    handlers: Vec<Arc<dyn Dispatchable>>,
}

impl Dispatcher {
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
        }
    }

    pub fn register<T: Dispatchable + 'static>(&mut self, handler: Arc<T>) {
        log::info!(
            "Registering dispatch handler: {}",
            handler.dispatcher_name()
        );
        self.handlers.push(handler);
    }

    pub fn dispatch(&self, buf: &[u8]) {
        metrics::MESSAGES_RECEIVED.inc();

        match Envelope::decode(buf) {
            Ok(env) => {
                for handler in &self.handlers {
                    metrics::MESSAGES_DISPATCHED.inc();
                    if let Err(e) = handler.dispatch(&env) {
                        metrics::MESSAGES_FAILED_DISPATCH
                            .with_label_values(&[handler.dispatcher_name()])
                            .inc();
                        log::warn!(
                            "Dispatcher {} failed to handle message: {}",
                            handler.dispatcher_name(),
                            e
                        );
                    }
                }
            }
            Err(e) => log::warn!("Failed to decode message: {}", e),
        }
    }
}
