use std::{any::Any, collections::HashMap, marker::PhantomData, sync::Arc};

use async_trait::async_trait;
use serde::Serialize;
use serde_json::Value;

use crate::ax::ActionHandler;

pub struct ActionDispatcher {
    context: Box<dyn AxContext>,
    action_handlers: HashMap<String, Vec<ActionHandler>>,
}

impl ActionDispatcher {
    pub fn new<T: AxContext>(context: T) -> Self {
        Self {
            context: Box::new(context),
            action_handlers: HashMap::new(),
        }
    }

    pub fn get_context<T: AxContext + 'static>(&self) -> &T {
        self.context.as_any().downcast_ref::<T>().unwrap()
    }

    pub fn add_action_handlers(&mut self, handlers: Vec<ActionHandler>) {
        let data_hooks = &mut self.action_handlers;
        for handler in handlers {
            let event_kind = handler.get_kind();
            let handlers = data_hooks.entry(event_kind.to_string()).or_insert(vec![]);
            handlers.push(handler);
        }
    }

    pub async fn trigger_action<T: AxEvent>(&self, action: T) -> Vec<AxResponse> {
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

            // Event Cascading : If responses contains event, send them back to the pipeline
            let mut next_futures = vec![];
            for response_entry in &res {
                if response_entry.success {
                    if let Some(next_event) = &response_entry.value {
                        let future = self.context.json_event(self, next_event);
                        next_futures.push(future);
                    }
                }
            }
            let _ = futures::future::join_all(next_futures).await;
            res
        } else {
            vec![]
        }
    }
    
}

#[async_trait]
pub trait AxContext: Any + Send + Sync {
    fn as_any(&self) -> &dyn Any;

    fn as_context(&self) -> &dyn AxContext;

    async fn json_event(&self, dispatcher: &ActionDispatcher, event: &Value) -> Vec<AxResponse>;
}

pub struct EventSchema<T> {
    pub name: &'static str,
    pub _marker: PhantomData<T>,
}

pub trait AxEvent: Any + Send + Sync {
    fn get_id(&self) -> &str;
    fn get_kind(&self) -> &str;
    fn get_key(&self) -> String;
}

#[derive(Debug, Serialize)]
pub struct AxResponse {
    pub success: bool,
    pub handler: String,
    pub value: Option<Value>,
}
