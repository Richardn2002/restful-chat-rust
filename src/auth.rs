use crate::db::Db;
use crate::jwt::gen_token;
use crate::types::UserId;

use bcrypt::verify;

use poem::web::Data;
use poem_openapi::{payload::Json, ApiResponse, Object, OpenApi};

enum LoginResult {
    Pass(UserId),
    UserNotFound,
    IncorrectPassword,
    MongoDbError,
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
    async fn login(&self, req: Json<LoginRequest>, db: Data<&Db>) -> LoginResponse {
        match check(db.0, &req.0.username, &req.0.password).await {
            LoginResult::Pass(uid) => {
                let api_key = gen_token(uid);

                LoginResponse::Ok(Json(LoginResponseBody { api_key }))
            }
            LoginResult::MongoDbError => LoginResponse::Error,
            LoginResult::BcryptError => LoginResponse::Error,
            _ => LoginResponse::Unauthorized,
        }
    }
}

async fn check(db: &Db, username: &str, password: &str) -> LoginResult {
    let (hash, uid) = match db.query_creds(username).await {
        Ok(opt) => match opt {
            Some(pair) => pair,
            None => return LoginResult::UserNotFound,
        },
        Err(_) => return LoginResult::MongoDbError,
    };

    match verify(password, &hash) {
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
