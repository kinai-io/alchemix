use std::{any::Any, sync::Arc};

use async_trait::async_trait;

use crate::{
    dispatcher::{DispatchPayload, Dispatcher, EntityAction}, entity::Entity, prelude::{EntitySchema, SQLiteEntityStore, SafeDataHookHandler, SafeSignalHookHandler}, rx::{RxAction, RxResponse}
};

use serde_json::Value;

#[async_trait]
pub trait RxContext {

    async fn update_entities(&self, store: &ReactiveStore, kind: &str, ids: Vec<Value>);

    async fn delete_entities(&self, store: &ReactiveStore, kind: &str, ids: &Vec<&str>);

    async fn get_entities(&self, store: &ReactiveStore, kind: &str, ids: &Vec<&str>) -> RxResponse;

    async fn query_property(&self, store: &ReactiveStore, kind: &str, property_name: &str, expression: &str) -> RxResponse;

}

pub type SafeContext = dyn Any + Send + Sync + 'static;

pub struct ReactiveStore {
    dispatcher: Dispatcher,
    store: SQLiteEntityStore,
    context: Box<SafeContext>,
}

impl ReactiveStore {
    pub fn new<T: Any + RxContext + Send + Sync + 'static>(context: T, path: &str) -> Self {
        Self {
            dispatcher: Dispatcher::new(),
            store: SQLiteEntityStore::new(path),
            context: Box::new(context),
        }
    }

    pub fn get_context<T: RxContext + 'static>(&self) -> &T {
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

    pub fn with_signal_hooks(mut self, hooks: Vec<Box<SafeSignalHookHandler>>) -> Self {
        self.dispatcher.register_signal_hooks(hooks);
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

    fn get_rx_context(&self) -> &dyn RxContext {
        *self.context.downcast_ref::<&dyn RxContext>().unwrap()
    }

    pub async fn execute_action(&self, action: RxAction) -> RxResponse {
        let rx_context = self.get_rx_context();
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
            RxAction::Signal(_signal) => {
                RxResponse::SignalResponse()
            }
        }
    }
}
