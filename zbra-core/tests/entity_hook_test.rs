use zbra::{dispatcher::{Context, Dispatcher, EntityAction}, prelude::*};
use zbra_flow_macros::{entity_update, entity_delete};

#[entity(index(name), index(rank))]
pub struct User {
    name: String,
    rank: usize,
    data: Vec<u8>,
}

#[entity_update(User)]
pub async fn on_save(context: &Context, value: &Vec<User>) {
    println!("Save users : {:?}", value);
}

#[entity_update(User)]
async fn long_save(context: &Context, value: &Vec<User>) {
    println!("long add : {:?}", value);
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    context.hello();
    println!("long add Complete");
}

#[entity_delete(User)]
async fn on_delete(context: &Context, value: &Vec<User>) {
    println!("Delete : {:?}", value);
}

#[tokio::test]
pub async fn test_hooks() {
    println!("Start");
    let mut dispatcher = Dispatcher::new();

    dispatcher.register_entity_hook(Box::new(OnSaveHandler));
    dispatcher.register_entity_hook(Box::new(LongSaveHandler));
    dispatcher.register_entity_hook(Box::new(OnDeleteHandler));
    
    let context = Arc::new(Context{});

    let user = User::new("u".to_string(), 1, vec![]);
    // Dispatch actions
    dispatcher.dispatch_entity_hook(context.clone(), EntityAction::Update, vec![user.clone()]).await;
    dispatcher.dispatch_entity_hook(context.clone(), EntityAction::Delete, vec![user]).await;
}