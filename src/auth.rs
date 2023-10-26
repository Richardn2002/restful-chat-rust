use crate::jwt::gen_token;
use crate::types::UserId;

use bcrypt::verify;

use poem_openapi::{payload::Json, ApiResponse, Object, OpenApi};

enum LoginResult {
    Pass(UserId),
    UserNotFound,
    IncorrectPassword,
    BcryptError,
}

#[derive(Object)]
struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Object)]
struct LoginResponseBody {
    api_key: String,
}

#[derive(ApiResponse)]
enum LoginResponse {
    #[oai(status = 200)]
    Ok(Json<LoginResponseBody>),
    // handled by poem
    // #[oai(status = 400)]
    // Invalid,
    #[oai(status = 401)]
    Unauthorized,
    #[oai(status = 500)]
    Error,
}

pub struct Auth;

#[OpenApi]
impl Auth {
    #[oai(path = "/login", method = "post")]
    async fn login(&self, req: Json<LoginRequest>) -> LoginResponse {
        match check(&req.0.username, &req.0.password).await {
            LoginResult::Pass(uid) => {
                let api_key = gen_token(uid);

                LoginResponse::Ok(Json(LoginResponseBody { api_key }))
            }
            LoginResult::BcryptError => LoginResponse::Error,
            _ => LoginResponse::Unauthorized,
        }
    }
}

async fn check(username: &String, password: &String) -> LoginResult {
    // dumb database
    let (hash, uid) = {
        if username == "admin" {
            // the password is "password"
            (
                "$2a$12$f7h31gSiD0e22vCLJss6ROst5kTYD3G0MIzyzpO9ef.FQ8W4Px3ee",
                0,
            )
        } else {
            return LoginResult::UserNotFound;
        }
    };

    match verify(password, hash) {
        Ok(correct) => {
            if correct {
                LoginResult::Pass(uid)
            } else {
                LoginResult::IncorrectPassword
            }
        }
        Err(_) => LoginResult::BcryptError,
    }
}
