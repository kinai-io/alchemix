use alchemix_rx::prelude::*;


#[entity]
pub struct DemoData {
    value: usize,
}

#[rx_context(DemoData)]
pub struct AppContext {}