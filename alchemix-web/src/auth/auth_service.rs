use alchemix_rx::prelude::*;
use alchemix_utils::{json_io, time};

use rocket::{catch, http::Status, post, serde::json::Json, State};
use serde::{Deserialize, Serialize};

use crate::{analytics::Analytics, auth::AuthChecker};

use super::UserInfo;

#[entity]
pub struct UserEnterEvent{
    pub user_id: String,
    pub success: bool
}

pub struct AuthService {
    pub db_path: String,
    pub server_secret: String,
    pub token_validity_in_hours: i64,
}
impl AuthService {
    pub fn new(db_path: &str, server_secret: &str, token_validity_in_hours: i64) -> Self {
        Self {
            db_path: db_path.to_string(),
            server_secret: server_secret.to_string(),
            token_validity_in_hours,
        }
    }

    pub fn find_user_from_login(&self, login: &str) -> Option<UserInfo> {
        let users: Vec<UserInfo> = json_io::load(&self.db_path).unwrap();
        let existing_user = users.into_iter().find(|u| &u.login == login);
        existing_user
    }

    pub fn find_user_from_id(&self, id: &str) -> Option<UserInfo> {
        let users: Vec<UserInfo> = json_io::load(&self.db_path).unwrap();
        let existing_user = users.into_iter().find(|u| u.get_id() == id);
        existing_user
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserCredentials {
    pub login: String,
    pub password: String,
}

#[entity]
pub struct UserAuth {
    full_name: String,
    token: String,
    roles: Vec<String>,
}

#[post("/login", format = "json", data = "<credentials>")]
pub fn login(
    credentials: Json<UserCredentials>,
    auth_service: &State<AuthService>,
    analytics: &State<Analytics>
) -> Result<Json<UserAuth>, Status> {

    let existing_user = auth_service.find_user_from_login(&credentials.login);

    match existing_user {
        Some(user) => {
            if user.is_password_valid(&credentials.password) {
                let secret = &auth_service.server_secret;
                let validity_in_hours = auth_service.token_validity_in_hours;
                let token = user.new_token(secret, validity_in_hours);
                analytics.log_event(UserEnterEvent::new(credentials.login.to_string(), true));
                return Ok(Json(UserAuth {
                    id: user.get_id().to_string(),
                    kind: "UserAuth".to_string(),
                    full_name: user.full_name.to_string(),
                    roles: user.roles.clone(),
                    token,
                }));
            }
        }
        _ => {},
    }
    analytics.log_event(UserEnterEvent::new(credentials.login.to_string(), false));
    Err(Status::Forbidden)
}

#[catch(403)]
pub fn forbidded_catcher(req: &rocket::Request) -> String {
    format!("Forbidden access: {}", req.uri())
}

#[post("/refresh_token")]
pub fn refresh_token(
    auth: AuthChecker,
    auth_service: &State<AuthService>,
    analytics: &State<Analytics>
) -> Result<Json<UserAuth>, Status> {
    let existing_user = auth_service.find_user_from_id(&auth.user_id);
    match existing_user {
        Some(user) => {
            let secret = &auth_service.server_secret;
            let validity_in_hours = auth_service.token_validity_in_hours;
            let token = user.new_token(secret, validity_in_hours);
            analytics.log_event(UserEnterEvent::new(auth.user_id.to_string(), true));
            return Ok(Json(UserAuth {
                id: user.get_id().to_string(),
                kind: "UserAuth".to_string(),
                full_name: user.full_name.to_string(),
                roles: user.roles.clone(),
                token,
            }));
        }
        _ => {},
    }
    analytics.log_event(UserEnterEvent::new(auth.user_id.to_string(), false));
    Err(Status::Forbidden)
}
