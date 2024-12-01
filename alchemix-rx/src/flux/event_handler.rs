use std::{future::Future, pin::Pin, sync::Arc};

use crate::{
    flux::Flux,
    prelude::*,
};

pub struct EventHandler {
    handler_func: Pin<Box<HandlerFunction>>,
    kind: String,
    handler_id: String,
}

impl EventHandler {
    pub fn new(
        handler_func: Pin<Box<HandlerFunction>>,
        kind: &str,
        handler_id: &str,
    ) -> EventHandler {
        EventHandler {
            handler_func,
            kind: kind.to_string(),
            handler_id: handler_id.to_string(),
        }
    }

    pub fn get_kind(&self) -> &str {
        &self.kind
    }

    pub fn get_action_id(&self) -> &str {
        &self.handler_id
    }

    pub async fn handle(&self, context: &Flux, value: Arc<Payload>) -> HookResponse {
        (self.handler_func)(context, value).await
    }

}

pub type HandlerFunction = fn(
    &Flux,
    Arc<Payload>,
) -> Pin<Box<dyn Future<Output = HookResponse> + Send + Sync + '_>>;



#[derive(Debug, Serialize)]
pub struct HookResponse {
    pub success: bool,
    pub handler: String,
    pub entities: Vec<Value>,
    pub message: String
}

impl HookResponse {

    pub fn set_handler(&mut self, handler: &str) {
        self.handler = handler.to_string();
    }

    pub fn error(message: &str)-> Self {
        Self {
            success: false,
            handler: "".to_string(),
            entities: vec![],
            message: message.to_string()
        }
    }

    pub fn ok()-> Self {
        Self {
            success: true,
            handler: "".to_string(),
            entities: vec![],
            message: "".to_string()
        }
    }

    pub fn entity<T: Entity>( entity: T)-> Self {
        Self {
            success: true,
            handler: "".to_string(),
            entities: vec![serde_json::to_value(entity).unwrap()],
            message: "".to_string()
        }
    }

    pub fn with_entity<T: Entity>(mut self, entity: T)-> Self {
       self.entities.push(serde_json::to_value(entity).unwrap());
       self
    }

}
