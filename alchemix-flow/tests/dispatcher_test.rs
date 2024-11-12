use alchemix_flow::prelude::*;

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

#[rx_context(User, TestEntity)]
pub struct AppContext{
    secret: String
}

impl AppContext {

    pub fn fake_op(&self) {
        println!("Context : Fake op -> {}", self.secret);
    }
}

async fn add(_context: Arc<DispatchPayload<'_>>, _value: Arc<Payload>) {
    println!("add");
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    // context.hello();
    
    println!("add Complete");

}

async fn long_add(_context: &DispatchPayload<'_>, value: &User) {
    println!("long add : {:?}", value);
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    println!("long add Complete");
}

async fn long_add_wrapper(context: Arc<DispatchPayload<'_>>, value: Arc<User>) {
    long_add(&context, &value).await;
}

async fn sub<'a>(context: Arc<DispatchPayload<'a>>, _value: Arc<Payload>) {
    context.store.save_entities(&vec![TestEntity::new(12)]).await;
}

pub struct MyAddHandler;

#[async_trait]
impl DataHookHandler for MyAddHandler {

    async fn handle(&self, context: Arc<DispatchPayload<'_>>, value: Arc<Payload>) {
        add(context, value).await;
    }

    fn get_action(&self) -> EntityAction {
        EntityAction::Update
    }

    fn get_entity_kind(&self) -> &str {
        "User"
    }
}

pub struct MyLongAddHandler;

#[async_trait]
impl DataHookHandler for MyLongAddHandler {

    async fn handle(&self, context: Arc<DispatchPayload<'_>>, value: Arc<Payload>){
        if let Ok(data) = value.downcast::<User>() {
            long_add_wrapper(context, data).await;
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

#[async_trait]
impl DataHookHandler for MySubtractHandler {
    async fn handle(&self, context: Arc<DispatchPayload<'_>>, value: Arc<Payload>){
        println!("sub");
        sub(context.clone(), value.clone()).await;
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

    let context = AppContext{
        secret: "internal secret".to_string()
    };

    let db_path = "test-data/out/entity-store.db";
    let store = ReactiveStore::new(context, db_path);

    let context = Arc::new(DispatchPayload::new(&store));

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
