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
    pub value: Option<Value>,
    pub message: String
}

impl HookResponse {
    pub fn error(handler: &str, message: &str)-> Self {
        Self {
            success: false,
            handler: handler.to_string(),
            value: None,
            message: message.to_string()
        }
    }

    pub fn ok(handler: &str)-> Self {
        Self {
            success: true,
            handler: handler.to_string(),
            value: None,
            message: "".to_string()
        }
    }

    pub fn entity<T: Entity>(handler: &str, entity: T)-> Self {
        Self {
            success: true,
            handler: handler.to_string(),
            value: Some(serde_json::to_value(entity).unwrap()),
            message: "".to_string()
        }
    }

}
