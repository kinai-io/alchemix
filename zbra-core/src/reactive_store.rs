use std::sync::Arc;

use tokio::sync::RwLock;

use crate::{
    dispatcher::{Context, Dispatcher, EntityAction},
    entity::Entity,
    prelude::{DataHookHandler, SQLiteEntityStore, SafeDataHookHandler},
};

pub struct ReactiveStore {
    dispatcher: RwLock<Dispatcher>,
    store: RwLock<SQLiteEntityStore>,
}

impl ReactiveStore {
    pub fn new(path: &str) -> Self {
        Self {
            dispatcher: RwLock::new(Dispatcher::new()),
            store: RwLock::new(SQLiteEntityStore::new(path)),
        }
    }

    pub async fn open(&self) {
        let mut store = self.store.write().await;
        let _ = store.open().await;
    }

    pub async fn close(&self) {
        let store = self.store.read().await;
        let _ = store.close().await;
    }

    pub async fn add_entity_hooks(&self, hooks: Vec<Box<SafeDataHookHandler>>) {
        let mut dispatcher = self.dispatcher.write().await;
        dispatcher.register_entity_hooks(hooks);
    }

    pub async fn save_entities<'a, T: Entity>(&'a self, entities: Vec<T>) {
        let ids: Vec<&str> = entities.iter().map(|e| e.get_id()).collect();
        println!("STORE : save_entities => {:?}", &ids);

        let context = Arc::new(Context::new(self));
        let store = self.store.read().await;
        store.update_entities(&entities).await;
        let dispatcher = self.dispatcher.read().await;
        dispatcher
            .dispatch_entity_hook(context, EntityAction::Update, entities)
            .await;
    }

    pub async fn delete_entities<'a, T: Entity>(&'a self, entities: Vec<T>) {
        let ids: Vec<&str> = entities.iter().map(|e| e.get_id()).collect();
        println!("STORE : delete_entities => {:?}", &ids);

        let context = Arc::new(Context::new(self));
        let keys: Vec<String> = entities.iter().map(|e| e.get_key()).collect();
        let keys_str = keys.iter().map(|e| e.as_str()).collect();
        let store = self.store.read().await;
        store.remove_entities(&keys_str).await;
        let dispatcher = self.dispatcher.read().await;
        dispatcher
            .dispatch_entity_hook(context, EntityAction::Delete, entities)
            .await;
    }

}
