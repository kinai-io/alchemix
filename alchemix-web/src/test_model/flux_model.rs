use alchemix_rx::prelude::*;

#[flux_event]
pub struct AddAction {
    pub left: u16,
    pub right: u16,
}

#[flux_event]
pub struct Sum {
    pub result: u16,
}

#[flux_hook]
pub async fn add_action(
    action: &AddAction,
    _dispatcher: &Flux,
    context: &AdderContext,
) -> Result<Sum, String> {
    let res = action.left + action.right;
    context.log(&format!(
        "Add: {} + {} = {}",
        action.left, action.right, res
    ));
    Ok(Sum::new(res))
}

#[flux_hook]
pub async fn sum_history(
    action: &Sum,
    _dispatcher: &Flux,
    _context: &AdderContext,
) -> Result<(), String> {
    println!("SUM History: {}", action.result);

    Ok(())
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