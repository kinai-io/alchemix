use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::entity_store::Entity;

// {"UpdateEntities":["DemoData",[{"id":"9c682bbb-fa84-4d7f-8e4e-d40ea8cd11df","kind":"DemoData","value":42}]]}

#[derive(Debug, Serialize, Deserialize)]
pub enum RxAction {
    UpdateEntities(String, Value),
    DeleteEntities(String, Vec<String>),
    QueryIds(String, Vec<String>),
    QueryProperty(String, String, String),
    Signal(Value),
}

impl RxAction {

    pub fn new_update_action<P: Entity>(kind: &str, entities: &Vec<P>) -> Self {
        let value = serde_json::to_value(entities).unwrap();
        RxAction::UpdateEntities(kind.to_string(), value)
    }

    pub fn new_delete_action(kind: &str, ids: Vec<String>) -> Self {
        RxAction::DeleteEntities(kind.to_string(), ids)
    }

    pub fn new_query_ids(kind: &str, ids: Vec<String>) -> Self{
        RxAction::QueryIds(kind.to_string(), ids)
    }

    pub fn new_query_property(kind: &str, property_name: &str, expression: &str) -> Self{
        RxAction::QueryProperty(kind.to_string(), property_name.to_string(), expression.to_string())
    }

    pub fn new_signal<P: Entity>(signal: P) -> Self{
        let value = serde_json::to_value(signal).unwrap();
        RxAction::Signal(value)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum RxResponse {
    Success(),
    QueryResponse(Value),
    SignalResponse(Value),
    Failure(String)
}