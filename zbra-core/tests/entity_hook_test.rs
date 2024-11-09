use zbra::prelude::*;

#[entity(index(name), index(rank))]
pub struct User {
    name: String,
    rank: usize,
    data: Vec<u8>,
}

#[entity]
pub struct TestEntity {
    value: usize,
}

#[entity_update(User)]
pub async fn on_save(_context: &Context<'_>, value: &Vec<User>) {
    println!("Save users : {:?}", value);
}

#[entity_update(User)]
async fn long_save(context: &Context<'_>, value: &Vec<User>) {
    println!("long add : {:?}", value);
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    // context.hello();
    context.store.save_entities(vec![TestEntity::new(12)]).await;
    println!("long add Complete");
}

#[entity_delete(User)]
async fn on_delete(_context: &Context<'_>, value: &Vec<User>) {
    println!("Delete : {:?}", value);
}

#[entity_update(TestEntity)]
async fn on_derive(_context: &Context<'_>, value: &Vec<TestEntity>) {
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    println!("On Derive : {:?}", value);
}

#[tokio::test]
pub async fn test_hooks() {
    println!("Start");
    let mut dispatcher = Dispatcher::new();
    dispatcher.register_entity_hooks(entity_hooks!(on_save, long_save, on_delete, on_derive));

    let db_path = "test-data/out/entity-store.db";
    let store = ReactiveStore::new(db_path);
    let context = Arc::new(Context::new(&store));

    let user = User::new("u".to_string(), 1, vec![]);

    // Dispatch actions
    dispatcher
        .dispatch_entity_hook(context.clone(), EntityAction::Update, vec![user.clone()])
        .await;

    dispatcher
        .dispatch_entity_hook(context.clone(), EntityAction::Delete, vec![user.clone()])
        .await;

}

#[tokio::test]
pub async fn test_reactive_store() {
    println!("Start");
    let db_path = "test-data/out/entity-store.db";

    let store = ReactiveStore::new(db_path)
        .with_entity_hooks(entity_hooks!(on_save, long_save, on_delete, on_derive))
        .open()
        .await;

    let user = User::new("u".to_string(), 1, vec![]);

    store.save_entities(vec![user.clone()]).await;

    store.delete_entities(vec![user.clone()]).await;

    store.close().await;
}
