use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::entity::Entity;

#[derive(Debug, Serialize, Deserialize)]
pub enum RxAction {
    UpdateEntities(String, Vec<Value>),
    DeleteEntities(String, Vec<String>),
    QueryIds(String, Vec<String>),
    QueryProperty(String, String, String),
    Signal(Value),
}

impl RxAction {

    pub fn new_update_action<P: Entity>(kind: &str, entities: &Vec<P>) -> Self {
        let values = entities
            .iter()
            .map(|e| serde_json::to_value(e).unwrap())
            .collect();
        RxAction::UpdateEntities(kind.to_string(), values)
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
}

#[derive(Debug)]
pub enum RxResponse {
    Success(),
    QueryResponse(),
    SignalResponse(),
    Failure(String)
}