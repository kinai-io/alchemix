use alchemix_rx::prelude::*;
use alchemix_utils::time;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub struct Analytics {}

impl Analytics {
    pub fn new() -> Self {
        Self {}
    }

    pub fn log_event<T: Entity>(&self, event: T) {
        let event = AppEvent {
            date: time::current_time_iso(),
            kind: event.get_kind().to_string(),
            content: serde_json::to_value(event).unwrap(),
        };
        // TODO Save Event
    }
}

#[derive(Serialize, Deserialize)]
pub struct AppEvent {
    date: String,
    kind: String,
    content: Value,
}
