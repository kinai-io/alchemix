// lib.rs

use std::any::Any;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex, RwLock};

use futures::executor::block_on;
use futures::join;

use crate::entity::Entity;
use crate::reactive_store::ReactiveStore;

pub type Payload = dyn Any + Send + Sync + 'static;
pub struct Context<'a> {
    store: &'a ReactiveStore,
}

impl<'a> Context<'a> {
    pub fn new(r: &'a ReactiveStore) -> Self {
        Self { store: r }
    }

    pub fn hello(&self) {
        println!("Hello");
    }

    pub async fn save_entities<T: Entity>(&self, entities: Vec<T>) {
        // let store = self.store.lock().unwrap();
        self.store.save_entities(entities).await
    }
}

pub trait DataHookHandler {
    fn handle<'a>(&'a self, context: Arc<Context<'a>>, value: Arc<Payload>) -> BoxFuture<'a, ()>;
    fn get_action(&self) -> EntityAction;
    fn get_entity_kind(&self) -> &str;
}

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + Sync + 'a>>;

pub struct Dispatcher {
    data_hooks: HashMap<String, Vec<Box<dyn DataHookHandler + Send + Sync + 'static>>>,
}

impl Dispatcher {
    pub fn new() -> Self {
        Dispatcher {
            data_hooks: HashMap::new(),
        }
    }

    pub fn register_entity_hooks(&mut self, hooks: Vec<Box<dyn DataHookHandler + Send + Sync>>) {
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
        context: Arc<Context<'a>>,
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

            println!("Dispatch : {}", action_key);

            let data_hooks = &self.data_hooks;
            if let Some(handlers) = data_hooks.get(&action_key) {
                let boxed = Arc::new(value);


                let mut futures: Vec<BoxFuture<'a, ()>> = vec![];
                for handler in handlers {
                    futures.push(handler.handle(context.clone(), boxed.clone()));
                }
                futures::future::join_all(futures).await;

                // for handler in handlers {
                //     block_on(handler.handle(context.clone(), boxed.clone()));
                // }
            }
        }
    }
}

unsafe impl Send for Dispatcher {}

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
