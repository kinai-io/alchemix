use std::{any::Any, collections::HashMap, sync::Arc};

use async_trait::async_trait;
use serde::Serialize;
use serde_json::Value;

type Payload = dyn Any + Send + Sync;

pub trait RxAction: Any + Send + Sync + 'static {
    fn get_id(&self) -> &str;
    fn get_kind(&self) -> &str;
}

#[derive(Debug, Serialize)]
pub struct RxResponse {
    pub success: bool,
    pub handler: String,
    pub value: Option<Value>,
}


#[async_trait]
pub trait ActionHandler {
    fn get_kind(&self) -> &str;
    fn get_action_id(&self) -> &str;
    async fn handle(&self, context:  &ActionDispatcher, value: Arc<Payload>) -> RxResponse;
}

pub type SafeActionHandler = dyn ActionHandler + Send + Sync;

#[async_trait]
pub trait ActionContext: Any + Send + Sync + 'static {

    fn as_any(&self) -> &dyn Any;

    fn as_context(&self) -> &dyn ActionContext;
}

pub struct ActionDispatcher {
    context: Box<dyn ActionContext>,
    action_handlers: HashMap<String, Vec<Box<SafeActionHandler>>>,
}

impl ActionDispatcher {
    pub fn new<T: ActionContext>(context: T) -> Self {
        Self {
            context: Box::new(context),
            action_handlers: HashMap::new(),
        }
    }

    pub fn get_context<T: ActionContext + 'static>(&self) -> &T {
        self.context.as_any().downcast_ref::<T>().unwrap()
    }

    pub fn add_action_handlers(&mut self, handlers: Vec<Box<SafeActionHandler>>) {
        let data_hooks = &mut self.action_handlers;
        for handler in handlers {
            let event_kind = handler.get_kind();
            let handlers = data_hooks.entry(event_kind.to_string()).or_insert(vec![]);
            handlers.push(handler);
        }
    }

    pub async fn trigger_action<T: RxAction>(&self, action: T) -> Vec<RxResponse> {
        let event_kind = action.get_kind();
        let data_hooks = &self.action_handlers;
        if let Some(handlers) = data_hooks.get(event_kind) {
            
            // let c = DispatchPayload::new(self, value_ref);
            let value = Arc::new(action);
            let mut futures = vec![];
            for handler in handlers {
                // handler.handle(context.clone(), value_ref.clone()).await;
                // let c = DispatchPayload::new(self, value_ref.clone());
                // let value_ref = Box::new(action.clone());
                let future = handler.handle(self, value.clone());
                futures.push(future);
            }
            let res = futures::future::join_all(futures).await;
            res
        } else {
            vec![]
        }
    }
}
