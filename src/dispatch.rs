use crate::metrics::MESSAGES_RECEIVED;
use crate::proto::homelabd::Envelope;

use prost::Message;

pub trait Dispatchable: Send + Sync {
    fn name(&self) -> &'static str;

    fn dispatch(&self, message: &Envelope);
}

pub struct Dispatcher {
    handlers: Vec<Box<dyn Dispatchable>>,
}

impl Dispatcher {
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
        }
    }

    pub fn register<T: Dispatchable + 'static>(&mut self, handler: T) {
        log::info!("Registering dispatch handler: {}", handler.name());
        self.handlers.push(Box::new(handler));
    }

    pub fn dispatch(&self, buf: &[u8]) {
        MESSAGES_RECEIVED.inc();

        match Envelope::decode(buf) {
            Ok(env) => {
                for handler in &self.handlers {
                    handler.dispatch(&env);
                }
            }
            Err(e) => log::warn!("Failed to decode message: {}", e),
        }
    }
}
