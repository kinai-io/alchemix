use std::sync::Arc;

use crate::{
    dispatcher::{Context, Dispatcher, EntityAction},
    entity::Entity,
    prelude::{EntitySchema, SQLiteEntityStore, SafeDataHookHandler},
};

pub struct ReactiveStore {
    dispatcher: Dispatcher,
    store: SQLiteEntityStore,
}

impl ReactiveStore {
    pub fn new(path: &str) -> Self {
        Self {
            dispatcher: Dispatcher::new(),
            store: SQLiteEntityStore::new(path),
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

        let context = Arc::new(Context::new(self));
        self.store.update_entities(&entities).await;
        self.dispatcher
            .dispatch_entity_hook(context, EntityAction::Update, entities)
            .await;
    }

    pub async fn delete_entities<T: Entity>(& self, kind: EntitySchema<T>, ids: &Vec<&str>) {
        let removed_entities: Vec<T> = self.store.remove_entities(&kind.name, ids).await;

        let context = Arc::new(Context::new(self));
        self.dispatcher
            .dispatch_entity_hook(context, EntityAction::Delete, removed_entities)
            .await;
    }

}
