use alchemix_web::{analytics::Analytics, auth::{self, AuthService}, spa_services::{self, SPA}};
use rocket::{
    catchers, get,
    http::{ContentType, Status},
    launch, post, routes,
    serde::json::Json,
    Build, Rocket, State,
};

pub struct AlchemixWeb {
    
}

impl AlchemixWeb {
    
    pub fn new() -> Self {
        Self {
            
        }
    }

    pub fn serve(&self) -> Rocket<Build> {
        
        let figment = rocket::Config::figment().merge(("address", "0.0.0.0"));
        let auth_service = AuthService::new("config/users.json", "default secret", 24 * 3);
        let analytics = Analytics::new();
        let spa_settings = SPA::default();
        
        rocket::custom(figment)
            .manage(spa_settings)
            .manage(auth_service)
            .manage(analytics)
            .mount("/", routes![spa_services::app_index, spa_services::app_resources])
            .register("/", catchers![auth::forbidded_catcher])
        // .mount(
        //     "/api",
        //     routes![
        //         auth::login,
        //         auth::refresh_token,
        //         analytics,
        //         hello,
        //         patients,
        //         patient_actions,
        //         get_page_image,
        //         synchronize_patient
        //     ],
        // )
    }
}

#[launch]
fn rocket() -> Rocket<Build> {
    AlchemixWeb::new().serve()
}
