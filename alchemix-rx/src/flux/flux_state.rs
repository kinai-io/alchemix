use crate::prelude::*;

pub struct FluxState {
    root_path: String,
}

impl FluxState {
    pub fn new(root_path: &str) -> Self {
        Self {
            root_path: root_path.to_string(),
        }
    }

    pub fn save<T: Entity>(&self, shard: &str, entities: &Vec<T>) {
        let store = self.get_store(shard);
        block_on(store.update_entities(entities));
        // WARNING: May hangs the app
        // block_on(store.close());
    }

    pub fn get_entities_of_kind<E: Entity>(
        &self,
        shard: &str,
        kind: &EntitySchema<E>,
        ids: &Vec<&str>,
    ) -> Vec<E> {
        let store = self.get_store(shard);
        let res = block_on(store.get_entities_of_kind(&kind.name, ids));
        // block_on(store.close());
        res
    }

    pub fn query_entities<E: Entity>(
        &self,
        shard: &str,
        kind: &EntitySchema<E>,
        property_name: &str,
        expr: &str,
    ) -> Vec<E> {
        let store = self.get_store(shard);
        let res = block_on(store.query_entities(&kind.name, property_name, expr));
        // block_on(store.close());
        res
    }

    fn get_store(&self, shard: &str) -> SQLiteEntityStore {
        let db_path = format!("{}/{}.db", self.root_path, shard);
        let store = SQLiteEntityStore::new(&db_path);
        store
    }
    
}

#[derive(Serialize, Deserialize)]
pub struct StateQuery {
    pub shard: String,
    pub kind: String,
    pub property_name: String,
    pub expr: String,
}

impl StateQuery {
    pub fn new(shard: &str, kind: &str, property_name: &str, expr: &str) -> Self {
        Self {
            shard: shard.to_string(),
            kind: kind.to_string(),
            property_name: property_name.to_string(),
            expr: expr.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct StateGetEntities {
    pub shard: String,
    pub kind: String,
    pub ids: Vec<String>,
}

impl StateGetEntities {
    pub fn new(shard: &str, kind: &str, ids: Vec<&str>) -> Self {
        Self {
            shard: shard.to_string(),
            kind: kind.to_string(),
            ids: ids.iter().map(|s| s.to_string()).collect(),
        }
    }
}


