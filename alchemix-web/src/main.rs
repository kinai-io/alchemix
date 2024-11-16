use alchemix_web::prelude::*;

#[launch]
async fn rocket() -> Rocket<Build> {
    let context = AppContext {};
    let mut rx_store = RxStore::new(context, "test-data/out/test.db");
    rx_store.open().await;
    AlchemixWeb::new().with_rx("demo", rx_store).serve()
}
