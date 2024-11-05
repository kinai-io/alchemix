use serde::{Deserialize, Serialize};
// use sqlx_demo::{entity_store::EntityStore, sqlite_datastore::SQLiteDatastore};
use ts_rs::TS;
use zbra::entity::{Entity, FieldIndex};
use zbra_macros::entity;
use uuid::Uuid;

#[entity(index(name))]
pub struct User {
    name: String,
    value: usize,
    data: Vec<u8>,
}

#[tokio::main]
async fn main() {
    let mut users = vec![];
    for i in 0..10 {
        let user = User::new_with_id(&format!("User #{}", i), format!("User #{}", i), i, format!("#{}", i).as_bytes().into());
        // let user = User::new(format!("User #{}", i), i, format!("#{}", i).as_bytes().into());
        users.push(user);
    }

    println!("users : {:?}", users);
}

// #[tokio::main]
// async fn main() -> Result<(), sqlx::Error> {
//     let create_tables_query = r#"
//             CREATE TABLE IF NOT EXISTS user (key TEXT not null PRIMARY KEY, value INTEGER, data BLOB not null);
//             CREATE TABLE IF NOT EXISTS nodes (key TEXT not null PRIMARY KEY, id TEXT not null, kind TEXT not null, derived INTEGER, data BLOB not null);
//             CREATE INDEX IF NOT EXISTS nodes_id ON nodes (id);
//             CREATE TABLE IF NOT EXISTS links (id TEXT not null PRIMARY KEY, predicate TEXT not null, source TEXT not null, target TEXT not null, ordering INTEGER, weight REAL);
//             CREATE TABLE IF NOT EXISTS properties (key TEXT not null PRIMARY KEY, id TEXT not null, kind TEXT not null, name TEXT not null, value TEXT );
//             CREATE INDEX IF NOT EXISTS properties_values ON properties (value);
//             "#;

//     let drop_tables_query = r#"
//             DROP TABLE nodes;
//             DROP TABLE links;
//             DROP TABLE properties;
//             DROP INDEX IF EXISTS nodes_id;
//             DROP INDEX IF EXISTS properties_values;
//             "#;

//     let datastore = SQLiteDatastore::new("./test-data/out/test.db")
//         .with_create_tables(create_tables_query)
//         .with_drop_tables(drop_tables_query)
//         .open()
//         .await;

//     match datastore {
//         Ok(datastore) => {
//             let mut users = vec![];
//             for i in 0..10 {
//                 users.push(User{
//                     key: format!("#{}", i),
//                     value: i,
//                     data: format!("#{}", i).as_bytes().into()
//                 });
//             }
//             let _ = datastore.execute_batch_insert("INSERT or REPLACE INTO user (data, key, value) VALUES (?, ?, ?)", users).await;

//             datastore.close().await;
//         }
//         Err(e) => {
//             println!("{:?}", e);
//         }
//     }

//     Ok(())
// }
