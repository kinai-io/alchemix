use sqlx::{migrate::MigrateDatabase, sqlite::SqlitePoolOptions, FromRow, Pool, Sqlite};

use crate::entity::{Entity, FieldIndex};

#[derive(FromRow)]
struct EntityData {
    data: Vec<u8>,
}

pub struct SQLiteEntityStore {
    pool: Option<Pool<Sqlite>>,
    max_pool: usize,
    path: String,
    pragmas: Option<String>,
}

impl SQLiteEntityStore {
    pub fn new(path: &str) -> Self {
        Self {
            pool: None,
            max_pool: 5,
            path: path.to_string(),
            pragmas: None,
        }
    }

    pub fn with_pragmas(mut self, pragmas: &str) -> Self {
        self.pragmas = Some(pragmas.to_string());
        self
    }

    pub async fn open(&mut self) -> Result<(), sqlx::Error> {
        let database_url = format!("sqlite:{}", &self.path);

        if !Sqlite::database_exists(&database_url)
            .await
            .unwrap_or(false)
        {
            println!("Creating database {}", &database_url);
            match Sqlite::create_database(&database_url).await {
                Ok(_) => println!("Create db success"),
                Err(error) => panic!("error: {}", error),
            }
        } else {
            println!("Database already exists");
        }

        let pool = SqlitePoolOptions::new()
            .max_connections(self.max_pool as u32)
            .connect(&database_url)
            .await?;

        self.pool = Some(pool);

        if let Some(pragmas) = &self.pragmas {
            self.execute_batch(pragmas).await;
        }
        self.create_tables().await;
        Ok(())
    }

    pub async fn update_entities<T: Entity>(&self, entities: &Vec<T>) {
        if let Some(pool) = &self.pool {
            let insert_sql_command =
                r#"INSERT or REPLACE INTO entity (key, id, kind, data) VALUES (?, ?, ?, ?)"#;
            let mut tx = pool.begin().await.unwrap();

            for entity in entities {
                if let Ok(entity_vec) = Self::entity_to_vec(entity) {
                    let _ = sqlx::query(insert_sql_command)
                        .bind(entity.get_key())
                        .bind(entity.get_id())
                        .bind(entity.get_kind())
                        .bind(entity_vec)
                        .execute(&mut *tx)
                        .await;
                } else {
                    println!("Unable to serialize entity")
                };
            }
            let _ = tx.commit().await;
            self.update_entities_index(entities).await;
        }
    }

    async fn update_entities_index<T: Entity>(&self, entities: &Vec<T>) {
        let fields_index: Vec<FieldIndex> = entities
            .iter()
            .map(|e| {
                let fields_index: Vec<FieldIndex> = e
                    .get_fields_index()
                    .into_iter()
                    .map(|field_index| field_index)
                    .collect();
                fields_index
            })
            .fold(vec![], |mut acc, mut index| {
                acc.append(&mut index);
                acc
            });

        let index_queries:Vec<String> = fields_index.iter().map(|fi| {
                let key = format!("{}#{}", fi.kind, fi.entity_id);
                let update_index_query = format!(
                    "INSERT or REPLACE INTO properties (key, id, kind, name, value) VALUES (\'{}\', \'{}\', \'{}\', \'{}\', \'{}\');",
                    key, fi.entity_id, fi.kind, fi.name, fi.value
                );
                update_index_query
            }).collect();
        let update_index_query = index_queries.join("\n");
        self.execute_batch(&update_index_query).await;
    }

    pub async fn remove_entities<T: Entity>(&self, kind: &str, ids: &Vec<&str>) -> Vec<T>{
        let stored_entities = self.get_entities_of_kind(kind, ids).await;

        let keys_strs: Vec<String> = ids.iter().map(|id| format!("\'{}#{}\'", kind, id)).collect();


        let keys_str = format!("({})", keys_strs.join(", "));

        let delete_entity_query = format!("DELETE from entity WHERE key IN {};", keys_str);

        let delete_properties_query = format!("DELETE from properties WHERE key IN {};", keys_str);

        let delete_query = format!("{}{}", delete_entity_query, delete_properties_query);
        self.execute_batch(&delete_query).await;
        stored_entities
    }

    async fn execute_batch(&self, sql_command: &str) {
        if let Some(pool) = &self.pool {
            let _ = sqlx::query(sql_command).execute(pool).await.unwrap();
        }
    }

    pub async fn get_entities_of_kind<E: Entity>(&self, kind: &str, ids: &Vec<&str>) -> Vec<E> {
        let sql_query = if ids.is_empty() {
            let query = format!("SELECT data FROM entity WHERE kind = \'{}\'", kind);
            query
        } else {
            let ids_str: Vec<String> = ids.iter().map(|k| format!("\'{}\'", k)).collect();
            let keys_str = format!("({})", ids_str.join(", "));
            let query = format!("SELECT data FROM entity WHERE id IN {}", keys_str);
            query
        };

        if let Some(pool) = &self.pool {
            let results: Vec<EntityData> =
                sqlx::query_as(&sql_query).fetch_all(pool).await.unwrap();
            Self::decode_entities(&results)
        } else {
            vec![]
        }
    }

    pub async fn query_entities<E: Entity>(
        &self,
        kind: &str,
        property_name: &str,
        expr: &str,
    ) -> Vec<E> {
        let sql_query = format!("SELECT data FROM entity WHERE key IN ( SELECT key FROM properties WHERE kind = \'{}\' AND name=\'{}' AND {})", kind, property_name, expr);
        if let Some(pool) = &self.pool {
            let results: Vec<EntityData> =
                sqlx::query_as(&sql_query).fetch_all(pool).await.unwrap();
            Self::decode_entities(&results)
        } else {
            vec![]
        }
    }

    async fn create_tables(&self) {
        let create_tables_query = r#"
            CREATE TABLE IF NOT EXISTS entity (key TEXT not null PRIMARY KEY, id TEXT not null, kind TEXT not null, data BLOB not null);
            CREATE INDEX IF NOT EXISTS nodes_id ON entity (id);
            CREATE TABLE IF NOT EXISTS links (id TEXT not null PRIMARY KEY, predicate TEXT not null, source TEXT not null, target TEXT not null, ordering INTEGER, weight REAL);
            CREATE TABLE IF NOT EXISTS properties (key TEXT not null PRIMARY KEY, id TEXT not null, kind TEXT not null, name TEXT not null, value TEXT );
            CREATE INDEX IF NOT EXISTS properties_values ON properties (value);
            "#;
        self.execute_batch(create_tables_query).await;
    }

    pub async fn clear(&self) {
        let drop_tables_query = r#"
            DROP TABLE entity;
            DROP TABLE links;
            DROP TABLE properties;
            DROP INDEX IF EXISTS nodes_id;
            DROP INDEX IF EXISTS properties_values;
            "#;
        self.execute_batch(drop_tables_query).await;
        self.create_tables().await;
    }

    pub async fn close(&self) {
        if let Some(pool) = &self.pool {
            pool.close().await;
        }
    }

    fn entity_to_vec<E: Entity>(entity: &E) -> Result<Vec<u8>, ()> {
        if let Ok(bytes) = bincode::serialize(entity) {
            Ok(bytes)
        } else {
            Err(())
        }
    }

    fn entity_from_vec<E: Entity>(data: &Vec<u8>) -> Result<E, ()> {
        let value = match bincode::deserialize(data) {
            Ok(data) => Ok(data),
            Err(_) => Err(()),
        };
        value
    }

    fn decode_entities<E: Entity>(data: &Vec<EntityData>) -> Vec<E> {
        let entities: Vec<E> = data
            .iter()
            .filter_map(|row| {
                let decoded = Self::entity_from_vec::<E>(&row.data);
                if let Ok(entity) = decoded {
                    Some(entity)
                } else {
                    None
                }
            })
            .collect();
        entities
    }
}
