use alchemix_rx::prelude::*;

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

#[entity]
pub struct CountUsers {
    _all: usize,
}

#[entity]
pub struct UsersSummary {
    count: usize,
}

#[rx_context(User, TestEntity, CountUsers, UsersSummary)]
pub struct AppContext {
    secret: String,
}

impl AppContext {
    pub fn fake_op(&self) {
        println!("Context : Fake op -> {}", self.secret);
    }
}

#[rx_entity_update(User)]
pub async fn on_save(value: &Vec<User>, _store: &RxStore) {
    println!("Save users : {:?}", value);
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
}

#[rx_entity_update(User)]
async fn long_save(value: &Vec<User>, store: &RxStore) {
    println!("long add : {:?}", value);
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    // context.hello();
    store
        .save_entities(&vec![TestEntity::new(value.len())])
        .await;
    println!("long add Complete");
}

#[rx_entity_delete(User)]
async fn on_delete(value: &Vec<User>, _store: &RxStore) {
    println!("Delete : {:?}", value);
}

#[rx_entity_update(TestEntity)]
async fn on_derive_data(value: &Vec<TestEntity>, _store: &RxStore, context: &AppContext) {
    // let context = store.get_context::<AppContext>();
    // context: &AppContext
    println!("On Derive : {:?}", value);

    context.fake_op();
    println!("context : {}", context.secret);
}

#[rx_signal_handler]
async fn count_users(
    _value: &CountUsers,
    store: &RxStore,
) -> Result<UsersSummary, String> {
    let res = store.get_entities(AppContext::USER, &vec![]).await;
    Ok(UsersSummary::new(res.len()))
}

#[entity]
pub struct AddUsers {
    users: Vec<User>,
}

#[rx_signal_handler]
async fn add_users(
    value: &AddUsers,
    store: &RxStore,
    _context: &AppContext,
) -> Result<AddUsers, String> {
    store.save_entities(&value.users).await;
    Ok(value.clone())
}

#[tokio::test]
pub async fn test_hooks() {
    println!("Start");
    let mut dispatcher = Dispatcher::new();
    dispatcher.register_entity_hooks(entity_hooks!(on_save, long_save, on_delete, on_derive_data));

    let context = AppContext {
        secret: "internal secret".to_string(),
    };
    let db_path = "test-data/out/entity-store.db";
    let store = RxStore::new(context, db_path);
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

fn create_users(index_start: usize, count: usize) -> Vec<User> {
    let mut new_users = vec![];
    for i in index_start..index_start + count {
        let user = User::new_with_id(
            &format!("User_{}", i),
            format!("User_{}", i),
            i,
            format!("#{}", i).as_bytes().into(),
        );
        new_users.push(user);
    }
    new_users
}

#[tokio::test]
pub async fn test_reactive_store() {
    println!("Start");
    let db_path = "test-data/out/entity-store.db";

    let context = AppContext {
        secret: "internal secret".to_string(),
    };

    
    let mut rx_store = RxStore::new(context, db_path)
        .with_entity_hooks(entity_hooks!(on_save, long_save, on_delete, on_derive_data))
        .with_signal_hooks(signal_hooks!(add_users, count_users));

    rx_store.open().await;
    rx_store.clear().await;

    let user = User::new("user_1".to_string(), 1, vec![]);

    rx_store.save_entities(&vec![user.clone()]).await;

    let users = rx_store.get_entities(AppContext::USER, &vec![]).await;
    assert_eq!(users.len(), 1);

    let new_users = create_users(0, 10);

    let _: Result<AddUsers, String> = rx_store.signal(AddUsers::new(new_users)).await;

    let users = rx_store.get_entities(AppContext::USER, &vec![]).await;
    assert_eq!(users.len(), 11);

    let count_result: Result<UsersSummary, String> = rx_store.signal(CountUsers::new(0)).await;
    println!("Signal output : {:?}", count_result);

    let res = rx_store
        .execute_action(RxAction::new_query_ids("User", vec![]))
        .await;
    println!("Query Action : {:?}", res);

    let new_users = create_users(100, 10);
    rx_store
        .execute_action(RxAction::new_update_action("User", &new_users))
        .await;

    let values = serde_json::to_value(users).unwrap();

    let _entities = serde_json::from_value::<Vec<User>>(values).unwrap();

    rx_store
        .delete_entities(AppContext::USER, &vec![user.id.as_str()])
        .await;

    let res = rx_store
        .execute_action(RxAction::new_query_property(
            "User",
            "rank",
            "value > 103 AND value < 106",
        ))
        .await;

    println!("Query Prop : {:?}", res);

    let res = rx_store
        .execute_action(RxAction::new_signal(CountUsers::new(0)))
        .await;
    println!("Action : {:?}", res);
    // rx_store.query_property(kind, property_name, expression)


    let out = serde_json::to_string(&RxAction::new_signal(CountUsers::new(0))).unwrap();
    
    let signal_data: RxAction = serde_json::from_str(&out).unwrap();

    println!("Json: {}", out);
    println!("Json Action: {:?}", &signal_data);

    rx_store.close().await;
}
