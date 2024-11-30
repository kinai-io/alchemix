use alchemix_rx::rx::{RxAction, RxResponse};
use rocket::tokio;

use crate::test_model::DemoData;

#[tokio::test]
pub async fn test_client() {
    println!("TEST");
    let url = "http://localhost:8000/api/demo/action";
    let client = reqwest::Client::new();

    let body = RxAction::new_update_action("DemoData", &vec![DemoData::new(42)]);
    let body_str = serde_json::to_string(&body).unwrap();
    println!("{}", body_str);

    let action = r#"
    {"id":"0ad24096-58a3-4a7b-90f1-f0016f4ca7bd","kind":"AddAction","left":2,"right":3}
    "#;

    // let res = client.post(url).body(body_str).send().await;

    // let body = RxAction::new_query_ids("DemoData", vec![]);
    // let body_str = serde_json::to_string(&body).unwrap();
    // let res = client.post(url).body(body_str).send().await;
    // if let Ok(response) = res {
    //     let response_body = response.json::<RxResponse>().await;
    //     if let Ok(rx_resp) = response_body {
    //         println!("{:?}", rx_resp);
    //     }
    // }
}
