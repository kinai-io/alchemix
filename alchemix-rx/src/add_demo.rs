use async_trait::async_trait;
use std::{any::Any, future::Future, pin::Pin, sync::Arc};

use crate::{
    action_dispatcher::{ActionContext, ActionDispatcher, ActionHandler, RxAction, RxResponse},
    prelude::Payload,
};
use serde::Serialize;

#[derive(Debug)]
pub struct AddAction {
    pub left: u16,
    pub right: u16,
}

impl RxAction for AddAction {
    fn get_id(&self) -> &str {
        "000"
    }

    fn get_kind(&self) -> &str {
        "AddAction"
    }
}

#[derive(Debug, Serialize)]
pub struct Sum {
    pub result: u16,
}

pub async fn execute_add(
    action: &AddAction,
    _dispatcher: &ActionDispatcher,
    context: &TestContext,
) -> Result<Sum, String> {
    let res = action.left + action.right;
    context.log(&format!(
        "Add: {} + {} = {}",
        action.left, action.right, res
    ));
    Ok(Sum { result: res })
}

pub fn handle_execute_add(
    dispatcher: &ActionDispatcher,
    value: Arc<Payload>,
) -> Pin<Box<dyn Future<Output = RxResponse> + Send + Sync + '_>> {
    let context: &TestContext = dispatcher.get_context();

    Box::pin(async move {
        // Simulate some work and return an RxResponse
        if let Ok(payload) = value.downcast::<AddAction>() {
            let p = payload.as_ref();
            let res = execute_add(p, dispatcher, context).await;
            if let Ok(res) = res {
               return  RxResponse {
                    success: true,
                    handler: "execute_add".to_string(),
                    value: Some(serde_json::to_value(res).unwrap()),
                }
            }
        }
        return RxResponse {
            success: false,
            handler: "execute_add".to_string(),
            value: None,
        };
    })

    //
}

pub type HandlerFunction = fn(&ActionDispatcher, Arc<Payload>) -> Pin<Box<dyn Future<Output = RxResponse> + Send + Sync + '_>>;

pub struct AddActionHandler {
    handler_func: Pin<Box<HandlerFunction>>,
}

impl AddActionHandler {
    pub fn new(handler_func: Pin<Box<HandlerFunction>>) -> Box<AddActionHandler> {
        Box::new(AddActionHandler { handler_func })
    }
}

#[async_trait]
impl ActionHandler for AddActionHandler {
    fn get_kind(&self) -> &str {
        "AddAction"
    }

    fn get_action_id(&self) -> &str {
        "execute_add"
    }

    async fn handle(&self, context: &ActionDispatcher, value: Arc<Payload>) -> RxResponse {
        (self.handler_func)(context, value).await
    }
}

pub struct TestContext {}

impl ActionContext for TestContext {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_context(&self) -> &dyn ActionContext {
        self
    }
}

impl TestContext {
    pub fn log(&self, text: &str) {
        println!("Log: {}", text);
    }
}
