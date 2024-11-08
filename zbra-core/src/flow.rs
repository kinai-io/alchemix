use serde::{Deserialize, Serialize};
use serde_json::Value;


#[derive(Debug, Serialize, Deserialize)]
pub struct Action {
    action: String,
    payload: Value,
}

impl Action {
    pub fn new<P: Serialize>(action: &str, payload: &P) -> Self {
        Self {
            action: action.to_string(),
            payload: serde_json::to_value(payload).unwrap(),
        }
    }
}


pub trait ZbraFlowContext {

}

pub struct ZbraFlow {
    // pub context: Box<dyn Any + 'static>,
    // data_handlers: HashMap<String, Vec<DataHandler>>,
}

impl ZbraFlow {

    pub fn new() -> Self {
        Self {
            // context: Box::new(context),
            // data_handlers: HashMap::new(),
        }
    }

    // pub fn on_update<T>(self, entity_type: EntitySchema<T>, handler: DataHandler) -> Self {
    //     let event_name = format!("update_{}", entity_type.name.to_ascii_lowercase());
    //     self.add_data_handler(&event_name, handler)
    // }

    // pub fn on_delete<T>(self, entity_type: EntitySchema<T>, handler: DataHandler) -> Self {
    //     let event_name = format!("delete_{}", entity_type.name.to_ascii_lowercase());
    //     self.add_data_handler(&event_name, handler)
    // }

    // pub fn add_data_handler(mut self, event: &str, handler: DataHandler) -> Self {
    //     let handlers = self.data_handlers.entry(event.to_string())
    //     .or_insert(vec![]);
    //     handlers.push(handler);
    //     self
    // }

    pub async fn trigger_data_handler<P: Serialize + Send + Sync>(& self, event: & str, payload: &P) {
        // self.context.disptach_event(&self, event, Box::new(payload)).await;
        // if let Some(handlers) = self.data_handlers.get(event) {
        //     for handler in handlers {
        //         let param = Box::new(payload as &(dyn Any + Send + Sync));
        //         handler(self, param).await;
        //     }
        // }
    }

    pub async fn execute(&self, action: Action) {
        match action.action.as_str() {
            "update" => {
                

            },
            "delete" => {

            }
            _ => {}
        }
    }
}

unsafe impl Send for ZbraFlow{}
unsafe impl Sync for ZbraFlow{}