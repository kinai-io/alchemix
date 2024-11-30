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

#[ax_context]
pub struct TestContext {}

impl TestContext {
    pub fn log(&self, text: &str) {
        println!("Log: {}", text);
    }
}

#[tokio::test]
pub async fn test_ax() {
    let context = TestContext {};
    let mut dispatcher = ActionDispatcher::new(context);

    dispatcher.add_action_handlers(event_hooks!(add_action));

    let action = AddAction::new(2, 3);

    let res = dispatcher.trigger_action(action).await;

    println!("Res : {:?}", res);
    println!("Res JSON : {:?}", serde_json::to_string(&res).unwrap());
}
