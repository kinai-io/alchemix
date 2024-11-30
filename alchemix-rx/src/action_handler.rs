use std::{future::Future, pin::Pin, sync::Arc};

use crate::{
    action_dispatcher::{ActionDispatcher, AxResponse},
    prelude::Payload,
};

pub struct ActionHandler {
    handler_func: Pin<Box<HandlerFunction>>,
    kind: String,
    handler_id: String,
}

impl ActionHandler {
    pub fn new(
        handler_func: Pin<Box<HandlerFunction>>,
        kind: &str,
        handler_id: &str,
    ) -> ActionHandler {
        ActionHandler {
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

    pub async fn handle(&self, context: &ActionDispatcher, value: Arc<Payload>) -> AxResponse {
        (self.handler_func)(context, value).await
    }

}

pub type HandlerFunction = fn(
    &ActionDispatcher,
    Arc<Payload>,
) -> Pin<Box<dyn Future<Output = AxResponse> + Send + Sync + '_>>;
