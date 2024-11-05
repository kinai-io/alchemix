use serde::Serialize;
use sqlx::{
    migrate::MigrateDatabase,
    sqlite::{SqliteArguments, SqlitePoolOptions},
    Arguments, Encode, Error, Pool, Sqlite, Type,
};

use crate::entity_store::EntityStore;

pub struct SQLiteDatastore {
    pool: Option<Pool<Sqlite>>,
    max_pool: usize,
    path: String,
    create_tables: Option<String>,
    drop_tables: Option<String>,
    pragmas: Option<String>,
}

impl SQLiteDatastore {
    pub fn new(path: &str) -> Self {
        Self {
            pool: None,
            max_pool: 5,
            path: path.to_string(),
            create_tables: None,
            drop_tables: None,
            pragmas: None,
        }
    }

    pub fn with_pragmas(mut self, pragmas: &str) -> Self {
        self.pragmas = Some(pragmas.to_string());
        self
    }

    pub fn with_create_tables(mut self, create_tables: &str) -> Self {
        self.create_tables = Some(create_tables.to_string());
        self
    }

    pub fn with_drop_tables(mut self, drop_tables: &str) -> Self {
        self.drop_tables = Some(drop_tables.to_string());
        self
    }

    pub async fn open(mut self) -> Result<Self, sqlx::Error> {
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
        if let Some(create_tables) = &self.create_tables {
            self.execute_batch(create_tables).await;
        }

        Ok(self)
    }

   pub  async fn execute_batch_insert<'q, T>(
        &self,
        sql_command: &str,
        rows: Vec<T>,
    ) -> Result<(), Error>
    where
        T: Serialize,
    {
        if let Some(pool) = &self.pool {
            let mut tx = pool.begin().await?;

            for row in rows {
                let mut sqlite_arguments = SqliteArguments::default();

                let json_value = serde_json::to_value(row).unwrap();

                let map_value = json_value.as_object().unwrap();

                for (key, value) in map_value {
                    sqlite_arguments.add(value).unwrap();
                }

                sqlx::query_with(sql_command, sqlite_arguments)
                    .execute(&mut *tx)
                    .await?;
            }

            tx.commit().await
        } else {
            Ok(())
        }
    }
}

impl EntityStore for SQLiteDatastore {
    async fn clear(&self) {
        if let Some(drop_tables) = &self.drop_tables {
            self.execute_batch(drop_tables).await;
        }
    }

    async fn close(&self) {
        if let Some(pool) = &self.pool {
            pool.close().await;
        }
    }

    async fn execute_batch(&self, sql_command: &str) {
        if let Some(pool) = &self.pool {
            let _ = sqlx::query(sql_command).execute(pool).await.unwrap();
        }
    }
}
