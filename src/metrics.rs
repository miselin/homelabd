use once_cell::sync::Lazy;
use prometheus::{IntCounter, IntCounterVec, Registry};

pub static REGISTRY: Lazy<Registry> = Lazy::new(Registry::new);

pub static MESSAGES_SENT: Lazy<IntCounter> = Lazy::new(|| {
    let m = IntCounter::new("messages_sent", "Messages sent via multicast").unwrap();
    REGISTRY.register(Box::new(m.clone())).unwrap();
    m
});

pub static MESSAGES_RECEIVED: Lazy<IntCounter> = Lazy::new(|| {
    let m = IntCounter::new("messages_received", "Messages received via multicast").unwrap();
    REGISTRY.register(Box::new(m.clone())).unwrap();
    m
});

pub static MESSAGES_DISPATCHED: Lazy<IntCounter> = Lazy::new(|| {
    let m = IntCounter::new("messages_dispatched", "Messages dispatched to handlers").unwrap();
    REGISTRY.register(Box::new(m.clone())).unwrap();
    m
});

pub static MESSAGES_FAILED_DISPATCH: Lazy<IntCounterVec> = Lazy::new(|| {
    let opts = prometheus::Opts::new(
        "messages_failed_dispatch",
        "Messages that failed to dispatch",
    );
    let m = IntCounterVec::new(opts, &["handler"]).unwrap();
    REGISTRY.register(Box::new(m.clone())).unwrap();
    m
});

pub fn gather() -> Vec<prometheus::proto::MetricFamily> {
    REGISTRY.gather()
}
