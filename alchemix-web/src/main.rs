use alchemix_rx::prelude::*;
use alchemix_web::{alchemix_web::AlchemixWeb, test_model::AppContext};

use rocket::{launch, Build, Rocket};


#[launch]
async fn rocket() -> Rocket<Build> {
    let context = AppContext {};
    let mut rx_store = RxStore::new(context, "test-data/out/test.db");
    rx_store.open().await;
    AlchemixWeb::new().with_rx("demo", rx_store).serve()
}
