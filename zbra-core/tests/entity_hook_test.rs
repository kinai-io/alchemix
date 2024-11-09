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


#[flow_context(User, TestEntity)]
pub struct AppContext{
    secret: String
}


impl AppContext {

    pub fn fake_op(&self) {
        println!("Context : Fake op -> {}", self.secret);
    }
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
async fn on_derive(value: &Vec<TestEntity>, _store: &ReactiveStore, context: &AppContext) {
    // let context = store.get_context::<AppContext>();
    // context: &AppContext
    println!("On Derive : {:?}", value);

    context.fake_op();
    println!("context : {}", context.secret);
}

#[tokio::test]
pub async fn test_hooks() {
    println!("Start");
    let mut dispatcher = Dispatcher::new();
    dispatcher.register_entity_hooks(entity_hooks!(on_save, long_save, on_delete, on_derive));

    let context = AppContext{
        secret: "internal secret".to_string()
    };
    let db_path = "test-data/out/entity-store.db";
    let store = ReactiveStore::new(context, db_path);
    let dispatch_payload = Arc::new(DispatchPayload::new(&store));

    let user = User::new("u".to_string(), 1, vec![]);

    // Dispatch actions
    dispatcher
        .dispatch_entity_hook(dispatch_payload.clone(), EntityAction::Update, vec![user.clone()])
        .await;

    dispatcher
        .dispatch_entity_hook(dispatch_payload.clone(), EntityAction::Delete, vec![user.clone()])
        .await;

}

#[tokio::test]
pub async fn test_reactive_store() {
    println!("Start");
    let db_path = "test-data/out/entity-store.db";

    let context = AppContext{
        secret: "internal secret".to_string()
    };
    let store = ReactiveStore::new(context, db_path)
        .with_entity_hooks(entity_hooks!(on_save, long_save, on_delete, on_derive))
        .open()
        .await;

    let user = User::new("user_1".to_string(), 1, vec![]);

    store.save_entities(vec![user.clone()]).await;
    store.delete_entities(AppContext::USER, &vec![user.id.as_str()]).await;

    store.close().await;
}
