use std::sync::Arc;

use zbra::{
    dispatcher::{BoxFuture, Context, DataHookHandler, Dispatcher, EntityAction, Payload},
    prelude::*,
};

#[entity(index(name), index(rank))]
pub struct User {
    name: String,
    rank: usize,
    data: Vec<u8>,
}

async fn add(context: Arc<Context>, value: Arc<Payload>) {
    println!("add");
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    // context.hello();
    println!("add Complete");
}

async fn long_add(context: &Context, value: &User) {
    println!("long add : {:?}", value);
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    context.hello();
    println!("long add Complete");
}

async fn long_add_wrapper(context: Arc<Context>, value: Arc<User>) {
    long_add(&context, &value).await;
}

async fn sub(context: Arc<Context>, value: Arc<Payload>) {
    context.hello();
}

pub struct MyAddHandler;

impl DataHookHandler for MyAddHandler {
    fn handle(&self, context: Arc<Context>, value: Arc<Payload>) -> BoxFuture<'static, ()> {
        let future = add(context, value);
        Box::pin(future)
    }

    fn get_action(&self) -> EntityAction {
        EntityAction::Update
    }

    fn get_entity_kind(&self) -> &str {
        "User"
    }
}

pub struct MyLongAddHandler;

impl DataHookHandler for MyLongAddHandler {
    fn handle(&self, context: Arc<Context>, value: Arc<Payload>) -> BoxFuture<'static, ()> {
        if let Ok(data) = value.downcast::<User>() {
            let future = long_add_wrapper(context, data);
            Box::pin(future)
        } else {
            Box::pin(noop())
        }
    }

    fn get_action(&self) -> EntityAction {
        EntityAction::Update
    }

    fn get_entity_kind(&self) -> &str {
        "User"
    }
}

pub struct MySubtractHandler;

impl DataHookHandler for MySubtractHandler {
    fn handle(&self, context: Arc<Context>, value: Arc<Payload>) -> BoxFuture<'static, ()> {
        println!("sub");
        let future = sub(context, value);
        Box::pin(future)
    }
    fn get_action(&self) -> EntityAction {
        EntityAction::Update
    }

    fn get_entity_kind(&self) -> &str {
        "User"
    }
}

#[tokio::test]
pub async fn test_dispatcher() {
    // Create a new Dispatcher instance
    let mut dispatcher = Dispatcher::new();

    // Create and register action handlers
    let add_handler = MyAddHandler;
    let subtract_handler = MySubtractHandler;

    dispatcher.register_entity_hooks(vec![
        Box::new(add_handler),
        Box::new(MyLongAddHandler),
        Box::new(subtract_handler),
    ]);

    let context = Arc::new(Context {});

    let user = User::new("u".to_string(), 1, vec![]);
    // Dispatch actions
    dispatcher
        .dispatch_entity_hook(context.clone(), EntityAction::Update, vec![user])
        .await;

    let user = User::new("u".to_string(), 1, vec![]);
    dispatcher
        .dispatch_entity_hook(context.clone(), EntityAction::Delete, vec![user])
        .await;
}
