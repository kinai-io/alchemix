use alchemix_web::{prelude::*, test_model::AdderContext};

#[launch]
async fn rocket() -> Rocket<Build> {
    let context = AppContext {};
    let mut rx_store = RxStore::new(context, "test-data/out/test.db");
    rx_store.open().await;

    let adder_flux = AdderContext {};

    AlchemixWeb::new()
        .with_rx("demo", rx_store)
        .with_flux("adder", adder_flux)
        .serve()
}
