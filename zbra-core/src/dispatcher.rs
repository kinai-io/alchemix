// lib.rs

use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;

use crate::entity::Entity;
use crate::reactive_store::ReactiveStore;


pub struct DispatchPayload<'a> {
    pub store: &'a ReactiveStore,
}

impl<'a> DispatchPayload<'a> {
    pub fn new(r: &'a ReactiveStore) -> Self {
        Self { store: r }
    }

    pub fn get_context<T: 'static>(&self) -> Option<&T> {
        self.store.get_context()
    }
}

pub type Payload = dyn Any + Send + Sync;

pub type SafeDataHookHandler = dyn DataHookHandler + Send + Sync;

#[async_trait]
pub trait DataHookHandler {
    async fn handle(&self, context: Arc<DispatchPayload<'_>>, value: Arc<Payload>);
    fn get_action(&self) -> EntityAction;
    fn get_entity_kind(&self) -> &str;
}

pub struct Dispatcher {
    data_hooks: HashMap<String, Vec<Box<SafeDataHookHandler>>>,
}

impl Dispatcher {
    pub fn new() -> Self {
        Dispatcher {
            data_hooks: HashMap::new(),
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
        let hooks: Vec<&str> = data_hooks.iter().map(|(k, _)| k.as_str()).collect();
        println!("Hooks : {:?}", hooks);
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
