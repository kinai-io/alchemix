use rocket::{
    http::Status,
    outcome::Outcome,
    request::{self, FromRequest, Request},
};

use super::{verify_token, AuthService};

pub struct AuthChecker {
    pub user_id: String,
    pub roles: Vec<String>
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthChecker {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let auth_service = request
            .rocket()
            .state::<AuthService>()
            .expect("AuthService managed state");

        match request.headers().get_one("Token") {
            Some(token) => match verify_token(token, &auth_service.server_secret) {
                Ok(claims) => {
                    Outcome::Success(AuthChecker {
                        user_id: claims.user_id,
                        roles: claims.roles
                    })
                }
                Err(_) => Outcome::Error((Status::Forbidden, ())),
            },
            None => Outcome::Error((Status::Forbidden, ())),
        }
    }
}