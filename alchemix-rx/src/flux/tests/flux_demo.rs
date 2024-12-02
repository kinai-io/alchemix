use alchemix_utils::file_io;

use crate::prelude::*;

#[entity]
pub struct AddAction {
    pub left: u16,
    pub right: u16,
}

#[entity]
pub struct Sum {
    pub left: u16,
    pub right: u16,
    pub result: u16,
}

#[flux_hook]
pub async fn add_action(
    action: &AddAction,
    _dispatcher: &FluxState,
    context: &TestContext,
) -> HookResponse {
    let res = action.left + action.right;
    context.log(&format!(
        "Add: {} + {} = {}",
        action.left, action.right, res
    ));
    HookResponse::entity(Sum::new(action.left, action.right, res))
}

#[flux_hook]
pub async fn sum_history(
    action: &Sum,
    state: &FluxState,
    _context: &TestContext,
) -> HookResponse {
    println!("SUM History: {}", action.result);
    state.save("default", &vec![action.clone()]);
    HookResponse::ok()
}

#[flux_context(
    events(AddAction, Sum),
    hooks(add_action, sum_history)
)]
pub struct TestContext {
    
}

impl TestContext {

    pub fn new() -> Self {
        Self {  }
    }

    pub fn log(&self, text: &str) {
        println!("Log: {}", text);
    }
}

#[tokio::test]
pub async fn test_flux() {
    let context = TestContext::new();
    
    let path = "test-data/out/flux-state";
    file_io::remove_dir(path);
    let flux = Flux::new(path, context);
    
    let action = AddAction::new(2, 3);

    let res = flux.push(action.clone()).await;

    println!("Res : {:?}", res);
    println!("Res JSON : {:?}", serde_json::to_string(&res).unwrap());

    println!("Action JSON : {}", serde_json::to_string(&action).unwrap());

    let query = StateQuery::new("default", "Sum", "result", "value == 5");
    let res = flux.query_entities(&query);
    println!("Query JSON : {}", serde_json::to_string(&res).unwrap());

    let query = StateGetEntities::new("default", "Sum", vec![]);
    let res = flux.get_entities(&query);
    println!("Query JSON : {}", serde_json::to_string(&res).unwrap());
    
}
