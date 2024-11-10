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
pub struct AppContext {
    secret: String,
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
    store.save_entities(&vec![TestEntity::new(value.len())]).await;
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

#[entity]
pub struct CountUsers {
    _all: usize,
}

#[entity]
pub struct UsersSummary {
    count: usize,
}

#[signal_handler]
async fn on_count_users(value: &CountUsers, store: &ReactiveStore) -> Result<UsersSummary, String> {
    let res = store.get_entities(AppContext::USER, &vec![]).await;
    Ok(UsersSummary::new(res.len()))
}

#[entity]
pub struct AddUsers {
    users: Vec<User>,
}

#[signal_handler]
async fn add_users(value: &AddUsers, store: &ReactiveStore, context: &AppContext) -> Result<AddUsers, String> {
    store.save_entities(&value.users).await;
    Ok(value.clone())
}

#[tokio::test]
pub async fn test_hooks() {
    println!("Start");
    let mut dispatcher = Dispatcher::new();
    dispatcher.register_entity_hooks(entity_hooks!(on_save, long_save, on_delete, on_derive));

    let context = AppContext {
        secret: "internal secret".to_string(),
    };
    let db_path = "test-data/out/entity-store.db";
    let store = ReactiveStore::new(context, db_path);
    let dispatch_payload = Arc::new(DispatchPayload::new(&store));

    let user = User::new("u".to_string(), 1, vec![]);

    // Dispatch actions
    dispatcher
        .dispatch_entity_hook(
            dispatch_payload.clone(),
            EntityAction::Update,
            vec![user.clone()],
        )
        .await;

    dispatcher
        .dispatch_entity_hook(
            dispatch_payload.clone(),
            EntityAction::Delete,
            vec![user.clone()],
        )
        .await;
}

#[tokio::test]
pub async fn test_reactive_store() {
    println!("Start");
    let db_path = "test-data/out/entity-store.db";

    let context = AppContext {
        secret: "internal secret".to_string(),
    };
    let mut store = ReactiveStore::new(context, db_path)
        .with_entity_hooks(entity_hooks!(on_save, long_save, on_delete, on_derive))
        .with_signal_hooks(signal_hooks!(add_users, on_count_users));

    store.open().await;
    store.clear().await;

    let user = User::new("user_1".to_string(), 1, vec![]);

    store.save_entities(&vec![user.clone()]).await;

    let users = store.get_entities(AppContext::USER, &vec![]).await;
    assert_eq!(users.len(), 1);

    let mut new_users = vec![];
    for i in 0..10 {
        let user = User::new_with_id(
            &format!("User_{}", i),
            format!("User_{}", i),
            i,
            format!("#{}", i).as_bytes().into(),
        );
        new_users.push(user);
    }

    let count_result: Result<AddUsers, String> = store.signal(AddUsers::new(new_users)).await;

    let users = store.get_entities(AppContext::USER, &vec![]).await;
    assert_eq!(users.len(), 11);

    let count_result: Result<UsersSummary, String> = store.signal(CountUsers::new(0)).await;
    println!("Signal output : {:?}", count_result);

    store
        .delete_entities(AppContext::USER, &vec![user.id.as_str()])
        .await;

    store.close().await;
}
