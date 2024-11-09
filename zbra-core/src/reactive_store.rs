use std::{any::Any, sync::Arc};

use crate::{
    dispatcher::{DispatchPayload, Dispatcher, EntityAction},
    entity::Entity,
    prelude::{EntitySchema, SQLiteEntityStore, SafeDataHookHandler},
};

pub type SafeContext = dyn Any + Send + Sync + 'static;

pub struct ReactiveStore {
    dispatcher: Dispatcher,
    store: SQLiteEntityStore,
    context: Option<Box<SafeContext>>,
}

impl ReactiveStore {

    pub fn new(path: &str) -> Self {
        Self {
            dispatcher: Dispatcher::new(),
            store: SQLiteEntityStore::new(path),
            context: None
        }
    }

    pub fn with_context<T: Send + Sync + 'static>(mut self, context: T) -> Self {
        self.context = Some(Box::new(context));
        self
    }

    pub fn get_context<T: 'static>(& self) -> Option<& T> {
        if let Some(c) = &self.context {
            c.downcast_ref()
        }else {
            None
        }
    }

    pub async fn open(mut self) -> Self{
        let _ = self.store.open().await;
        self
    }

    pub async fn close(&self) {
        let _ = self.store.close().await;
    }

    pub fn with_entity_hooks(mut self, hooks: Vec<Box<SafeDataHookHandler>>) -> Self{
        self.dispatcher.register_entity_hooks(hooks);
        self
    }
    
    

    pub async fn save_entities<'a, T: Entity>(&'a self, entities: Vec<T>) {
        let ids: Vec<&str> = entities.iter().map(|e| e.get_id()).collect();
        println!("STORE : save_entities => {:?}", &ids);

        let context = Arc::new(DispatchPayload::new(self));
        self.store.update_entities(&entities).await;
        self.dispatcher
            .dispatch_entity_hook(context, EntityAction::Update, entities)
            .await;
    }

    pub async fn delete_entities<T: Entity>(& self, kind: EntitySchema<T>, ids: &Vec<&str>) {
        let removed_entities: Vec<T> = self.store.remove_entities(&kind.name, ids).await;

        let context = Arc::new(DispatchPayload::new(self));
        self.dispatcher
            .dispatch_entity_hook(context, EntityAction::Delete, removed_entities)
            .await;
    }

}
