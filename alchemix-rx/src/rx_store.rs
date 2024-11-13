use std::{any::Any, sync::Arc};

use async_trait::async_trait;

use crate::{
    dispatcher::{DispatchPayload, Dispatcher, EntityAction}, entity::Entity, prelude::{EntitySchema, SQLiteEntityStore, SafeDataHookHandler, SafeSignalHookHandler}, rx::{RxAction, RxResponse}
};

use serde_json::Value;

#[async_trait]
pub trait RxContext: Any + Send + Sync + 'static{

    fn as_any(&self) -> &dyn Any;

    async fn update_entities(&self, store: &RxStore, kind: &str, ids: Value);

    async fn delete_entities(&self, store: &RxStore, kind: &str, ids: &Vec<&str>);

    async fn get_entities(&self, store: &RxStore, kind: &str, ids: &Vec<&str>) -> RxResponse;

    async fn query_property(&self, store: &RxStore, kind: &str, property_name: &str, expression: &str) -> RxResponse;

    async fn signal(&self, store: &RxStore, signal: Value) -> RxResponse;

}

pub struct RxStore {
    dispatcher: Dispatcher,
    store: SQLiteEntityStore,
    context: Box<dyn RxContext>,
}

impl RxStore {
    pub fn new<T: RxContext>(context: T, path: &str) -> Self {
        Self {
            dispatcher: Dispatcher::new(),
            store: SQLiteEntityStore::new(path),
            context: Box::new(context),
        }
    }

    pub fn get_context<T: RxContext + 'static>(&self) -> &T {
        self.context.as_any().downcast_ref::<T>().unwrap()
    }

    pub async fn open(&mut self) {
        let _ = self.store.open().await;
    }

    pub async fn clear(&self) {
        let _ = self.store.clear().await;
    }

    pub async fn close(&self) {
        let _ = self.store.close().await;
    }

    pub fn with_entity_hooks(mut self, hooks: Vec<Box<SafeDataHookHandler>>) -> Self {
        self.dispatcher.register_entity_hooks(hooks);
        self
    }

    pub fn with_signal_hooks(mut self, hooks: Vec<Box<SafeSignalHookHandler>>) -> Self {
        self.dispatcher.register_signal_hooks(hooks);
        self
    }

    pub async fn save_entities<'e, T: Entity>(&'e self, entities: &Vec<T>) {
        let context = Arc::new(DispatchPayload::new(self));
        self.store.update_entities(entities).await;
        self.dispatcher
            .dispatch_entity_hook(context, EntityAction::Update, entities.clone())
            .await;
    }

    pub async fn delete_entities<T: Entity>(&self, kind: EntitySchema<T>, ids: &Vec<&str>) {
        let removed_entities: Vec<T> = self.store.remove_entities(&kind.name, ids).await;
        let context = Arc::new(DispatchPayload::new(self));
        self.dispatcher
            .dispatch_entity_hook(context, EntityAction::Delete, removed_entities)
            .await;
    }

    pub async fn get_entities<T: Entity>(&self, kind: EntitySchema<T>, ids: &Vec<&str>) -> Vec<T> {
        self.store.get_entities_of_kind(&kind.name, ids).await
    }

    pub async fn query_property<T: Entity>(&self, kind: EntitySchema<T>, property_name: &str, expression: &str) -> Vec<T> {
        self.store.query_entities(&kind.name, property_name, expression).await
    }

    pub async fn signal<T:Entity, R: Entity>(&self, signal_entity: T) -> Result<R, String> {
        let context = Arc::new(DispatchPayload::new(self));
        self.dispatcher.dispatch_signal_hook(context, signal_entity).await
    }

    pub async fn signal_action<T:Entity>(&self, signal_entity: T) -> Result<Value, String> {
        let context = Arc::new(DispatchPayload::new(self));
        self.dispatcher.dispatch_signal_action(context, signal_entity).await
    }

    pub async fn execute_action(&self, action: RxAction) -> RxResponse {
        let rx_context = &self.context;
        match action {
            RxAction::UpdateEntities(kind, values) => {
                rx_context.update_entities(&self, &kind, values).await;
                RxResponse::Success()
            }
            RxAction::DeleteEntities(kind, ids) => {
                let ids_ref = ids.iter().map(|id| id.as_str()).collect();
                rx_context.delete_entities(&self, &kind, &ids_ref).await;
                RxResponse::Success()
            }
            RxAction::QueryIds(kind, ids) => {
                let ids_ref = ids.iter().map(|id| id.as_str()).collect();
                rx_context.get_entities(&self, &kind, &ids_ref).await
            }
            RxAction::QueryProperty(kind, property_name, expression) => {
                rx_context.query_property(&self, &kind, &property_name, &expression).await
            }
            RxAction::Signal(signal) => {
                rx_context.signal(&self, signal).await
            }
        }
    }
}
