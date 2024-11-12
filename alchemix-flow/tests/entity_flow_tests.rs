use tokio::time::sleep;
use alchemix_flow::prelude::*;

#[entity(index(name), index(rank))]
pub struct User {
    name: String,
    rank: usize,
    data: Vec<u8>,
}

use std::{any::Any, time::Duration};

// #[entity_hook]
pub async fn increment_users_count(context: &UserFlow, users: &Vec<User>) {
    println!("increment_users_count : {:?}", users.len());
    println!("-> secret : {}", context.secret);

    sleep(Duration::from_secs(2)).await;
    println!("increment_users_count : Done");
}

pub async fn long_process_users(_context: &UserFlow, users: &Vec<User>) {
    println!("long_process_users : {:?}", users.len());
    sleep(Duration::from_secs(4)).await;
    println!("long_process_users : Done");
}

// #[entity_hook]
pub async fn decrement_users_count(context: &UserFlow, users: &Vec<User>) {
    println!("decrement_users_count : {:?}", users.len());
    println!("-> secret : {}", context.secret);
}

#[rx_context(User)]
pub struct UserFlow {
    secret: String,
}

impl UserFlow {
    pub fn new() -> Self {
        Self {
            secret: "hello".to_string(),
        }
    }

    async fn update_entities<T: Entity>(&self, entities: &Vec<T>) {
        let kind = if let Some(entity) = entities.first() {
            Some(entity.get_kind())
        } else {
            None
        };
        if let Some(kind) = kind {
            let event_name = format!("update_{}", kind.to_ascii_lowercase());
            self.trigger_data_handler(&event_name, entities.clone())
                .await;
        }
    }

    async fn trigger_data_handler<P: Serialize + Send + Sync + 'static>(
        &self,
        event: &str,
        payload: P,
    ) {
        self.disptach_event(event, Box::new(payload)).await;
    }

    async fn disptach_event(&self, event: &str, data: Box<(dyn Any + Sync + Send)>) {
        match event {
            "update_user" => {
                if let Some(data) = data.downcast_ref::<Vec<User>>() {
                    tokio::join!(
                        increment_users_count(self, data),
                        long_process_users(self, data)
                    );
                } else {
                    println!("Downcast error");
                }
            }
            "delete_user" => {
                if let Some(data) = data.downcast_ref::<Vec<User>>() {
                    decrement_users_count(self, data).await;
                } else {
                    println!("Downcast error");
                }
            }
            _ => {}
        }
    }
}

#[tokio::test]
pub async fn test_entity_flow() {
    let mut users = vec![];

    for i in 0..10 {
        let user = User::new_with_id(
            &format!("User_{}", i),
            format!("User_{}", i),
            i,
            format!("#{}", i).as_bytes().into(),
        );
        users.push(user);
    }

    let user_flow = UserFlow::new();
    user_flow.update_entities(&users).await;

    // user_flow_context
    //     .trigger_data_handler("update_user", &users)
    //     .await;

    // user_flow_context
    //     .trigger_data_handler("delete_user", &users)
    //     .await;
    // println!("{:?}", users);
}
