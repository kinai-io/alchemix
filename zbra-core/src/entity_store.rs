use crate::entity::Entity;

pub trait EntityStore {

    async fn clear(&self);

    async fn close(&self);

    async fn update_entity<E: Entity + 'static>(&self, entity: E);
    
}


