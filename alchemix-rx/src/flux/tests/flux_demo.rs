use crate::prelude::*;

#[flux_event]
pub struct AddAction {
    pub left: u16,
    pub right: u16,
}

#[flux_event]
pub struct Sum {
    pub left: u16,
    pub right: u16,
    pub result: u16,
}

#[flux_hook]
pub async fn add_action(
    action: &AddAction,
    _dispatcher: &Flux,
    context: &TestContext,
) -> Result<Sum, String> {
    let res = action.left + action.right;
    context.log(&format!(
        "Add: {} + {} = {}",
        action.left, action.right, res
    ));
    Ok(Sum::new(action.left, action.right, res))
}

#[flux_hook]
pub async fn sum_history(
    action: &Sum,
    _dispatcher: &Flux,
    context: &TestContext,
) -> Result<(), String> {
    println!("SUM History: {}", action.result);
    block_on(context.entity_store.update_entities(&vec![action.clone()]));
    Ok(())
}

#[flux_context(
    events(AddAction, Sum),
    hooks(add_action, sum_history)
)]
pub struct TestContext {
    entity_store: SQLiteEntityStore
}

impl TestContext {

    pub fn new(path: &str) -> Self {
        Self { entity_store: SQLiteEntityStore::new(path) }
    }

    pub fn log(&self, text: &str) {
        println!("Log: {}", text);
    }
}

#[tokio::test]
pub async fn test_flux() {
    let context = TestContext::new("test-data/out/flux-demo/test.db");
    let dispatcher = Flux::new(context);
    
    let action = AddAction::new(2, 3);

    let res = dispatcher.dispatch_event(action.clone()).await;

    println!("Res : {:?}", res);
    println!("Res JSON : {:?}", serde_json::to_string(&res).unwrap());

    println!("Action JSON : {}", serde_json::to_string(&action).unwrap());
}
