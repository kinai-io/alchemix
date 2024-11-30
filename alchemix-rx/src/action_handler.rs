use async_trait::async_trait;
use std::{future::Future, pin::Pin, sync::Arc};

use crate::{
    action_dispatcher::{ActionDispatcher, ActionHandler, RxResponse},
    prelude::Payload,
};

pub struct DefaultActionHandler {
    handler_func: Pin<Box<HandlerFunction>>,
    kind: String,
    handler_id: String,
}

impl DefaultActionHandler {
    pub fn new(
        handler_func: Pin<Box<HandlerFunction>>,
        kind: &str,
        handler_id: &str,
    ) -> Box<DefaultActionHandler> {
        Box::new(DefaultActionHandler {
            handler_func,
            kind: kind.to_string(),
            handler_id: handler_id.to_string(),
        })
    }
}

#[async_trait]
impl ActionHandler for DefaultActionHandler {
    fn get_kind(&self) -> &str {
        &self.kind
    }

    fn get_action_id(&self) -> &str {
        &self.handler_id
    }

    async fn handle(&self, context: &ActionDispatcher, value: Arc<Payload>) -> RxResponse {
        (self.handler_func)(context, value).await
    }
}

pub type HandlerFunction = fn(
    &ActionDispatcher,
    Arc<Payload>,
) -> Pin<Box<dyn Future<Output = RxResponse> + Send + Sync + '_>>;
