use rocket::{
    catchers, fairing::AdHoc, http::Status, post, routes, serde::json::Json, Build, Rocket, State,
};

use std::collections::HashMap;

use alchemix_rx::prelude::*;

use crate::{
    analytics::Analytics,
    auth::{self, AuthService},
    spa_services::{self, SPA},
};

pub struct AlchemixWeb {
    rx_stores: HashMap<String, RxStore>,
    fluxes: HashMap<String, Flux>,
}

impl AlchemixWeb {
    pub fn new() -> Self {
        Self {
            rx_stores: HashMap::new(),
            fluxes: HashMap::new(),
        }
    }

    pub fn with_flux<T: FluxContext>(mut self, name: &str, flux_context: T) -> Self {
        self.fluxes
            .insert(name.to_string(), Flux::new(flux_context));
        self
    }

    #[deprecated]
    pub fn with_rx(mut self, name: &str, rx: RxStore) -> Self {
        self.rx_stores.insert(name.to_string(), rx);
        self
    }

    pub fn serve(self) -> Rocket<Build> {
        let figment = rocket::Config::figment().merge(("address", "0.0.0.0"));
        let auth_service = AuthService::new("config/users.json", "default secret", 24 * 3);
        let analytics = Analytics::new();
        let spa_settings = SPA::default();

        rocket::custom(figment)
            .manage(spa_settings)
            .manage(auth_service)
            .manage(analytics)
            .manage(self)
            .mount(
                "/",
                routes![spa_services::app_index, spa_services::app_resources],
            )
            .register("/", catchers![auth::forbidded_catcher])
            .mount(
                "/api",
                routes![auth::login, auth::refresh_token, rx_action_post, flux_post],
            )
            .attach(AdHoc::on_shutdown("Shutdown Printer", |_| {
                Box::pin(async move {
                    println!("...shutdown has commenced!");
                    // TODO : https://rocket.rs/guide/v0.5/fairings/#callbacks
                })
            }))
    }

    #[deprecated]
    pub fn get_rx(&self, name: &str) -> Option<&RxStore> {
        self.rx_stores.get(name)
    }

    pub fn get_flux(&self, name: &str) -> Option<&Flux> {
        self.fluxes.get(name)
    }
}

#[post("/rx/<rx_name>/action", data = "<action>")]
pub async fn rx_action_post(
    rx_name: &str,
    action: Json<RxAction>,
    alchemix_web: &State<AlchemixWeb>,
    _analytics: &State<Analytics>,
) -> Result<Json<RxResponse>, Status> {
    if let Some(rx) = alchemix_web.get_rx(rx_name) {
        let response = rx.execute_action(action.0).await;
        Ok(Json(response))
    } else {
        Ok(Json(RxResponse::Success()))
    }
}

#[post("/flux/<flux_name>/event", data = "<event>")]
pub async fn flux_post(
    flux_name: &str,
    event: Json<Value>,
    alchemix_web: &State<AlchemixWeb>,
    _analytics: &State<Analytics>,
) -> Result<Json<Value>, Status> {
    if let Some(flux) = alchemix_web.get_flux(flux_name) {
        let response = flux.push_json(event.0).await;
        Ok(Json(serde_json::to_value(response).unwrap()))
    } else {
        Err(Status::ServiceUnavailable)
    }
}
