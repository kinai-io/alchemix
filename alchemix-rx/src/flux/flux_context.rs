use std::any::Any;

use async_trait::async_trait;
use serde_json::Value;

use super::{Flux, AxResponse, EventHandler};

#[async_trait]
pub trait FluxContext: Any + Send + Sync {
    fn as_any(&self) -> &dyn Any;

    fn as_context(&self) -> &dyn FluxContext;

    fn get_hooks(&self) -> Vec<EventHandler>;

    async fn json_event(&self, dispatcher: &Flux, event: &Value) -> Vec<AxResponse>;
}