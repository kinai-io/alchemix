// lib.rs

use std::any::Any;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use crate::entity::Entity;

pub type Payload = dyn Any + Send + Sync + 'static;
pub struct Context {}

impl Context {

    pub fn new() -> Self {
        Self {}
    }

    pub fn hello(&self) {
        println!("Hello");
    }
    
}


pub trait DataHookHandler {
    fn handle(&self, context: Arc<Context>, value: Arc<Payload>) -> BoxFuture<'static, ()>;
    fn get_action(&self) -> EntityAction;
    fn get_entity_kind(&self) -> &str;
}

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub struct Dispatcher {
    data_hooks: HashMap<String, Vec<Box<dyn DataHookHandler>>>,
}

impl Dispatcher {
    pub fn new() -> Self {
        Dispatcher {
            data_hooks: HashMap::new(),
        }
    }

    pub fn register_entity_hooks(&mut self, hooks: Vec<Box<dyn DataHookHandler>>) {
        for handler in hooks {
        let action = handler.get_action();
        let entity_kind = handler.get_entity_kind();

        let action_id = format!("{}_{}", action.get_text(), entity_kind);
        let handlers = self.data_hooks.entry(action_id).or_insert(vec![]);
        handlers.push(handler);
        }
    }

    pub async fn dispatch_entity_hook<T: Entity>(
        &self,
        context: Arc<Context>,
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
            if let Some(handlers) = self.data_hooks.get(&action_key) {
                let boxed = Arc::new(value);
                let mut futures: Vec<BoxFuture<'static, ()>> = vec![];
                for handler in handlers {
                    futures.push(handler.handle(context.clone(), boxed.clone()));
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
    Delete
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
