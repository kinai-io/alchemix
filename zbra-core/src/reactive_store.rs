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
    context: Box<SafeContext>,
}

impl ReactiveStore {
    pub fn new<T: Send + Sync + 'static>(context: T, path: &str) -> Self {
        Self {
            dispatcher: Dispatcher::new(),
            store: SQLiteEntityStore::new(path),
            context: Box::new(context),
        }
    }

    pub fn get_context<T: 'static>(&self) -> &T {
        &self.context.downcast_ref().unwrap()
    }

    pub async fn open(mut self) -> Self {
        let _ = self.store.open().await;
        self
    }

    pub async fn close(&self) {
        let _ = self.store.close().await;
    }

    pub fn with_entity_hooks(mut self, hooks: Vec<Box<SafeDataHookHandler>>) -> Self {
        self.dispatcher.register_entity_hooks(hooks);
        self
    }

    pub async fn save_entities<'a, T: Entity>(&'a self, entities: Vec<T>) {
        let context = Arc::new(DispatchPayload::new(self));
        self.store.update_entities(&entities).await;
        self.dispatcher
            .dispatch_entity_hook(context, EntityAction::Update, entities)
            .await;
    }

    pub async fn delete_entities<T: Entity>(&self, kind: EntitySchema<T>, ids: &Vec<&str>) {
        let removed_entities: Vec<T> = self.store.remove_entities(&kind.name, ids).await;

        let context = Arc::new(DispatchPayload::new(self));
        self.dispatcher
            .dispatch_entity_hook(context, EntityAction::Delete, removed_entities)
            .await;
    }
}
