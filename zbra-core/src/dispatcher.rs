// lib.rs

use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;

use crate::entity::Entity;
use crate::prelude::RxContext;
use crate::reactive_store::ReactiveStore;

pub struct DispatchPayload<'a> {
    pub store: &'a ReactiveStore,
}

impl<'a> DispatchPayload<'a> {
    pub fn new(r: &'a ReactiveStore) -> Self {
        Self { store: r }
    }

    pub fn get_context<T: RxContext + 'static>(&self) -> &T {
        self.store.get_context()
    }
}

pub type Payload = dyn Any + Send + Sync;

pub type SafeDataHookHandler = dyn DataHookHandler + Send + Sync;

pub type SafeSignalHookHandler = dyn SignalHookHandler + Send + Sync;

#[async_trait]
pub trait SignalHookHandler {
    async fn handle(
        &self,
        context: Arc<DispatchPayload<'_>>,
        value: Arc<Payload>,
    ) -> Result<Box<Payload>, String>;

    fn get_name(&self) -> &str;
}

#[async_trait]
pub trait DataHookHandler {
    async fn handle(&self, context: Arc<DispatchPayload<'_>>, value: Arc<Payload>);
    fn get_action(&self) -> EntityAction;
    fn get_entity_kind(&self) -> &str;
}

pub struct Dispatcher {
    data_hooks: HashMap<String, Vec<Box<SafeDataHookHandler>>>,
    signal_hook: HashMap<String, Box<SafeSignalHookHandler>>,
}

impl Dispatcher {
    pub fn new() -> Self {
        Dispatcher {
            data_hooks: HashMap::new(),
            signal_hook: HashMap::new(),
        }
    }

    pub fn register_entity_hooks(&mut self, hooks: Vec<Box<SafeDataHookHandler>>) {
        let data_hooks = &mut self.data_hooks;
        for handler in hooks {
            let action = handler.get_action();
            let entity_kind = handler.get_entity_kind();

            let action_id = format!("{}_{}", action.get_text(), entity_kind);

            let handlers = data_hooks.entry(action_id).or_insert(vec![]);
            handlers.push(handler);
        }
        // let hooks: Vec<&str> = data_hooks.iter().map(|(k, _)| k.as_str()).collect();
        // println!("Hooks : {:?}", hooks);
    }

    pub fn register_signal_hooks(&mut self, hooks: Vec<Box<SafeSignalHookHandler>>) {
        for handler in hooks {
            self.signal_hook.insert(handler.get_name().to_string(), handler);
        }
    }

    pub async fn dispatch_signal_hook<'a, T: Entity, R: Entity>(
        &'a self,
        context: Arc<DispatchPayload<'a>>,
        signal_entity: T,
    ) -> Result<R, String>{
        if let Some(handler) =  self.signal_hook.get(signal_entity.get_kind()) {
            let value_ref = Arc::new(signal_entity);
            let response = handler.handle(context, value_ref).await;
            match response {
                Ok(data) => {
                    if let Ok(data) = data.downcast::<R>() {
                        Ok(*data)
                    }else {
                        Err("Downcast error".to_string())
                    }
                },
                Err(message) => Err(message)
            }
        }else {
            Err("Unable to find signal handler".to_string())
        }
    }

    pub async fn dispatch_entity_hook<'a, T: Entity>(
        &'a self,
        context: Arc<DispatchPayload<'a>>,
        action: EntityAction,
        value: Vec<T>,
    ) {
        let entity_kind = if let Some(entity) = value.first() {
            Some(entity.get_kind().to_string())
        } else {
            None
        };
        if let Some(entity_kind) = entity_kind {
            let action_key = format!("{}_{}", action.get_text(), entity_kind);
            let data_hooks = &self.data_hooks;
            if let Some(handlers) = data_hooks.get(&action_key) {
                let value_ref = Arc::new(value);
                let mut futures = vec![];
                for handler in handlers {
                    // handler.handle(context.clone(), value_ref.clone()).await;
                    let future = handler.handle(context.clone(), value_ref.clone());
                    futures.push(future);
                }
                futures::future::join_all(futures).await;
            }
        }
    }
}

const UPDATE_ENTITY_ACTION: &str = "update";
const DELETE_ENTITY_ACTION: &str = "delete";

pub enum EntityAction {
    Update,
    Delete,
}

impl EntityAction {
    pub fn get_text(&self) -> &str {
        match self {
            EntityAction::Update => UPDATE_ENTITY_ACTION,
            EntityAction::Delete => DELETE_ENTITY_ACTION,
        }
    }
}

pub async fn noop() {}
