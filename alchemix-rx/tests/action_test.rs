
use std::pin::Pin;

use alchemix_rx::{
    action_dispatcher::ActionDispatcher,
    add_demo::{handle_execute_add, AddAction, AddActionHandler, HandlerFunction, TestContext}, prelude::Payload,
};

#[tokio::test]
pub async fn test_action() {
    
    let context = TestContext {};
    let mut dispatcher = ActionDispatcher::new(context);

    
    
    let handler:Box<HandlerFunction>  = Box::new(handle_execute_add);
    
    dispatcher.add_action_handlers(vec![AddActionHandler::new(Pin::new(handler))]);

    
    let add_action = AddAction { left: 2, right: 3 };
    let res = dispatcher.trigger_action(add_action).await;

    println!("Res : {:?}", res);
    println!("Res JSON : {:?}", serde_json::to_string(&res).unwrap());
}
