use std::sync::{Arc, Mutex};

use crate::{dispatcher::{Context, Dispatcher, EntityAction}, entity::Entity, prelude::{DataHookHandler, SQLiteEntityStore}};

pub struct ReactiveStore {
    dispatcher: Dispatcher,
    store: SQLiteEntityStore
}

impl ReactiveStore {
    
    pub fn new() -> Self{
        Self {
            dispatcher: Dispatcher::new(),
            store: SQLiteEntityStore::new("test-data/out/entity-store.db")
        }
    }

    pub fn with_entity_hooks(mut self, hooks: Vec<Box<dyn DataHookHandler>>) -> Self {
        self.dispatcher.register_entity_hooks(hooks);
        self
    }

    pub async fn save_entities<T:Entity>(&self, entities: Vec<T>) {
        let context = Arc::new(Context::new());
        
        self.store.update_entities(&entities).await;
        self.dispatcher.dispatch_entity_hook(context, EntityAction::Update, entities).await;
    }

    pub async fn delete_entities<T:Entity>(&self, entities: Vec<T>) {
        let context = Arc::new(Context::new());
        let keys: Vec<String> = entities.iter().map(|e| e.get_key()).collect();
        let keys_str = keys.iter().map(|e| e.as_str()).collect();
        self.store.remove_entities(&keys_str).await;
        self.dispatcher.dispatch_entity_hook(context, EntityAction::Delete, entities).await;
    }

}
