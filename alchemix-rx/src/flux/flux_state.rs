use crate::prelude::*;

pub struct FluxState {
    root_path: String
}

impl FluxState {

    pub fn new(root_path: &str) -> Self {
        Self {
            root_path: root_path.to_string()
        }
    }

    pub fn save<T: Entity>(&self, shard: &str, entities: &Vec<T>) {
        let store = self.get_store(shard);
        block_on(store.update_entities(entities));
        block_on(store.close());
    }

    pub fn get_entities_of_kind<E: Entity>(&self, shard: &str, kind: &EntitySchema<E>, ids: &Vec<&str>) -> Vec<E> {
        let store = self.get_store(shard);
        let res = block_on(store.get_entities_of_kind(&kind.name, ids));
        block_on(store.close());
        res
    }

    pub async fn query_entities<E: Entity>(
        &self,
        shard: &str,
        kind: &EntitySchema<E>,
        property_name: &str,
        expr: &str,
    ) -> Vec<E> {
        let store = self.get_store(shard);
        let res = block_on(store.query_entities(&kind.name, property_name, expr));
        block_on(store.close());
        res
    }

    fn get_store(&self, shard: &str) -> SQLiteEntityStore{
        let db_path = format!("{}/{}.db", self.root_path, shard);
        let store = SQLiteEntityStore::new(&db_path);
        store
    }

}