use std::sync::Arc;

// use futures::executor::block_on;
use zbra::{
    dispatcher::{BoxFuture, Context, DataHookHandler, Dispatcher, EntityAction, Payload},
    prelude::*, reactive_store::ReactiveStore,
};

#[entity(index(name), index(rank))]
pub struct User {
    name: String,
    rank: usize,
    data: Vec<u8>,
}

#[entity]
pub struct TestEntity {
    value: usize
}


async fn add(context: Arc<Context<'_>>, value: Arc<Payload>) {
    println!("add");
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    // context.hello();
    
    println!("add Complete");

}

async fn long_add(context: &Context<'_>, value: &User) {
    println!("long add : {:?}", value);
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    context.hello();
    println!("long add Complete");
}

async fn long_add_wrapper(context: Arc<Context<'_>>, value: Arc<User>) {
    long_add(&context, &value).await;
}

async fn sub<'a>(context: Arc<Context<'a>>, value: Arc<Payload>) {
    context.hello();
    context.save_entities(vec![TestEntity::new(12)]).await;
}

pub struct MyAddHandler;

impl DataHookHandler for MyAddHandler {
    fn handle<'a>(&'a self, context: Arc<Context<'a>>, value: Arc<Payload>) -> BoxFuture<'a, ()> {
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
    fn handle<'a>(&'a self, context: Arc<Context<'a>>, value: Arc<Payload>) -> BoxFuture<'a, ()> {
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
    fn handle<'a>(&'a self, context: Arc<Context<'a>>, value: Arc<Payload>) -> BoxFuture<'a, ()> {
        println!("sub");
        
        block_on(sub(context.clone(), value.clone()));
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

    let store = ReactiveStore::new();
    let context = Arc::new(Context::new(&store));

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
