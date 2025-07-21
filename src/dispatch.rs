use crate::metrics::MESSAGES_RECEIVED;
use crate::proto::homelabd::Envelope;

use prost::Message;

pub fn dispatch(buf: &[u8]) {
    MESSAGES_RECEIVED.inc();

    match Envelope::decode(buf) {
        Ok(env) => log::info!("Received message: {:?}", env),
        Err(e) => log::warn!("Failed to decode message: {}", e),
    }
}
