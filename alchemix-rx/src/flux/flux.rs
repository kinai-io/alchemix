use std::{collections::HashMap, marker::PhantomData, sync::Arc};

use serde_json::Value;

use crate::{flux::EventHandler, prelude::Entity};

use super::{FluxContext, FluxState, HookResponse};

pub struct Flux {
    state: FluxState,
    context: Box<dyn FluxContext>,
    action_handlers: HashMap<String, Vec<EventHandler>>,
}

impl Flux {
    pub fn new<T: FluxContext>(root_path: &str, context: T) -> Self {
        let hooks = context.get_hooks();
        let mut instance = Self {
            state: FluxState::new(root_path),
            context: Box::new(context),
            action_handlers: HashMap::new(),
        };
        instance.add_action_handlers(hooks);
        instance
    }

    pub fn get_context<T: FluxContext + 'static>(&self) -> &T {
        self.context.as_any().downcast_ref::<T>().unwrap()
    }

    pub fn get_state(&self) -> &FluxState {
        &self.state
    }

    fn add_action_handlers(&mut self, handlers: Vec<EventHandler>) {
        let data_hooks = &mut self.action_handlers;
        for handler in handlers {
            let event_kind = handler.get_kind();
            let handlers = data_hooks.entry(event_kind.to_string()).or_insert(vec![]);
            handlers.push(handler);
        }
    }

    pub async fn push<T: Entity>(&self, action: T) -> Vec<HookResponse> {
        let event_kind = action.get_kind();
        let data_hooks = &self.action_handlers;
        if let Some(handlers) = data_hooks.get(event_kind) {
            let value = Arc::new(action);
            let mut futures = vec![];
            for handler in handlers {
                let future = handler.handle(self, value.clone());
                futures.push(future);
            }
            let mut res = futures::future::join_all(futures).await;

            // Event Cascading : If responses contains event, send them back to the pipeline
            let mut next_futures = vec![];
            for response_entry in &res {
                if response_entry.success {
                    for next_event in &response_entry.entities {
                        let future = self.context.json_event(self, next_event);
                        next_futures.push(future);
                    }
                }
            }
            let all_reponses = futures::future::join_all(next_futures).await;
            all_reponses.into_iter().for_each(|response | {
                res.extend(response);
            });
            res
        } else {
            vec![]
        }
    }

    pub async fn push_json(&self, event: Value) -> Vec<HookResponse> {
        self.context.json_event(self, &event).await
    }
}
