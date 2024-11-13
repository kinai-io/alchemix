use std::time::SystemTime;

use chrono::{DateTime, Utc};

pub fn system_time_to_iso(time: SystemTime) -> String {
    let time: DateTime<Utc> = time.into();
    time.to_rfc3339()
}


pub fn current_time_iso() -> String {
    let current_time = SystemTime::now();
    let time: DateTime<Utc> = current_time.into();
    time.to_rfc3339()
}