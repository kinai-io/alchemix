use std::{future::Future, pin::Pin, sync::Arc};

use crate::{
    flux::{Flux, AxResponse},
    prelude::Payload,
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

    pub async fn handle(&self, context: &Flux, value: Arc<Payload>) -> AxResponse {
        (self.handler_func)(context, value).await
    }

}

pub type HandlerFunction = fn(
    &Flux,
    Arc<Payload>,
) -> Pin<Box<dyn Future<Output = AxResponse> + Send + Sync + '_>>;
