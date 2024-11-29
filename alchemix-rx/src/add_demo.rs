use std::{any::Any, sync::Arc};
use async_trait::async_trait;

use crate::action_dispatcher::{ActionContext, ActionDispatcher, ActionHandler, DispatchPayload, RxAction, RxResponse};
use serde::Serialize;


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

pub async fn execute_add(action: &AddAction, _dispatcher: &ActionDispatcher, context: &TestContext) -> Result<Sum, String> {
    let res = action.left + action.right;
    context.log(&format!(
        "Add: {} + {} = {}",
        action.left, action.right, res
    ));
    Ok(Sum { result: res })
}

pub struct AddActionHandler {}


#[async_trait]
impl ActionHandler for AddActionHandler {
    fn get_kind(&self) -> &str {
        "AddAction"
    }

    fn get_action_id(&self) -> &str {
        "execute_add"
    }

    async fn handle(&self, dispatch_payload: Arc<DispatchPayload<'_>>) -> RxResponse {
        let context : &TestContext= dispatch_payload.get_context();
        let payload : &AddAction = dispatch_payload.value.downcast_ref().unwrap();
        let res = execute_add(payload, dispatch_payload.dispatcher, context).await;
        if let Ok(res) = res {
            RxResponse {
                success: true,
                handler: self.get_action_id().to_string(),
                value: Some(serde_json::to_value(res).unwrap()),
            }
        }else {
            RxResponse {
                success: false,
                handler: self.get_action_id().to_string(),
                value: None,
            }
        }
        
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