use serde::{Deserialize, Serialize};
use serde_json::Value;


#[derive(Debug, Serialize, Deserialize)]
pub struct Action {
    action: String,
    payload: Value,
}

impl Action {
    pub fn new<P: Serialize>(action: &str, payload: &P) -> Self {
        Self {
            action: action.to_string(),
            payload: serde_json::to_value(payload).unwrap(),
        }
    }
}

