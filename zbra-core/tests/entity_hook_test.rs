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
pub async fn on_save(value: &Vec<User>, _store: &ReactiveStore) {
    println!("Save users : {:?}", value);
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
}

#[entity_update(User)]
async fn long_save(value: &Vec<User>, store: &ReactiveStore) {
    println!("long add : {:?}", value);
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    // context.hello();
    store.save_entities(vec![TestEntity::new(12)]).await;
    println!("long add Complete");
}

#[entity_delete(User)]
async fn on_delete(value: &Vec<User>, _store: &ReactiveStore) {
    println!("Delete : {:?}", value);
}

#[entity_update(TestEntity)]
async fn on_derive(value: &Vec<TestEntity>, store: &ReactiveStore, context: &AppContext) {
    // let context = store.get_context::<AppContext>();
    // context: &AppContext
    println!("On Derive : {:?}", value);

    println!("context : {}", context.secret);
}

#[tokio::test]
pub async fn test_hooks() {
    println!("Start");
    let mut dispatcher = Dispatcher::new();
    dispatcher.register_entity_hooks(entity_hooks!(on_save, long_save, on_delete, on_derive));

    let db_path = "test-data/out/entity-store.db";
    let store = ReactiveStore::new(db_path);
    let context = Arc::new(DispatchPayload::new(&store));

    let user = User::new("u".to_string(), 1, vec![]);

    // Dispatch actions
    dispatcher
        .dispatch_entity_hook(context.clone(), EntityAction::Update, vec![user.clone()])
        .await;

    dispatcher
        .dispatch_entity_hook(context.clone(), EntityAction::Delete, vec![user.clone()])
        .await;

}

#[flow_context(User)]
pub struct AppContext{
    secret: String
}

#[tokio::test]
pub async fn test_reactive_store() {
    println!("Start");
    let db_path = "test-data/out/entity-store.db";

    let store = ReactiveStore::new(db_path)
        .with_entity_hooks(entity_hooks!(on_save, long_save, on_delete, on_derive))
        .with_context(AppContext{
            secret: "internal secret".to_string()
        })
        .open()
        .await;

    let context: Option<&AppContext> = store.get_context();

    let user = User::new("user_1".to_string(), 1, vec![]);

    store.save_entities(vec![user.clone()]).await;

    store.delete_entities(AppContext::USER, &vec![user.id.as_str()]).await;

    store.close().await;
}
