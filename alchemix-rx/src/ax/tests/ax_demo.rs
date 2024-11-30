use crate::prelude::*;

#[ax_event]
pub struct AddAction {
    pub left: u16,
    pub right: u16,
}

#[ax_event]
pub struct Sum {
    pub result: u16,
}

#[ax_hook]
pub async fn add_action(
    action: &AddAction,
    _dispatcher: &ActionDispatcher,
    context: &TestContext,
) -> Result<Sum, String> {
    let res = action.left + action.right;
    context.log(&format!(
        "Add: {} + {} = {}",
        action.left, action.right, res
    ));
    Ok(Sum::new(res))
}

#[ax_hook]
pub async fn sum_history(
    action: &Sum,
    _dispatcher: &ActionDispatcher,
    _context: &TestContext,
) -> Result<(), String> {
    println!("SUM History: {}", action.result);

    Ok(())
}

#[ax_context(
    events(AddAction, Sum),
    hooks(add_action, sum_history)
)]
pub struct TestContext {}

impl TestContext {
    pub fn log(&self, text: &str) {
        println!("Log: {}", text);
    }
}

#[tokio::test]
pub async fn test_ax() {
    let context = TestContext {};
    let dispatcher = ActionDispatcher::new(context);
    let action = AddAction::new(2, 3);

    let res = dispatcher.dispatch_event(action).await;

    println!("Res : {:?}", res);
    println!("Res JSON : {:?}", serde_json::to_string(&res).unwrap());
}
