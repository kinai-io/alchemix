
use alchemix_rx::{
    action_dispatcher::ActionDispatcher,
    add_demo::{AddAction, AddActionHandler, TestContext},
};

#[tokio::test]
pub async fn test_action() {
    let context = TestContext {};
    let mut dispatcher = ActionDispatcher::new(context);

    dispatcher.add_action_handlers(vec![Box::new(AddActionHandler {})]);

    let add_action = AddAction { left: 2, right: 3 };
    let res = dispatcher.trigger_action(add_action).await;

    println!("Res : {:?}", res);
    println!("Res JSON : {:?}", serde_json::to_string(&res).unwrap());
}
