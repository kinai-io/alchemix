use alchemix_rx::prelude::*;

#[entity]
pub struct AddAction {
    pub left: u16,
    pub right: u16,
}

#[entity]
pub struct Sum {
    pub result: u16,
}

#[flux_hook]
pub async fn add_action(
    action: &AddAction,
    _dispatcher: &Flux,
    context: &AdderContext,
) -> HookResponse {
    let res = action.left + action.right;
    context.log(&format!(
        "Add: {} + {} = {}",
        action.left, action.right, res
    ));
    HookResponse::entity(Sum::new(res))
}

#[flux_hook]
pub async fn sum_history(
    action: &Sum,
    _dispatcher: &Flux,
    _context: &AdderContext,
) -> HookResponse {
    println!("SUM History: {}", action.result);

    HookResponse::ok()
}

#[flux_context(
    events(AddAction, Sum),
    hooks(add_action, sum_history)
)]
pub struct AdderContext {}

impl AdderContext {
    pub fn log(&self, text: &str) {
        println!("Log: {}", text);
    }
}