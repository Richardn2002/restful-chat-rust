#[macro_use]
extern crate lazy_static;

pub mod auth;
pub mod types;

use crate::auth::{check_token, gen_token, uid_from_token, ApiKeyAuthN};
use crate::types::*;

use dotenv::dotenv;
use std::env;

use std::error::Error;

use poem::{listener::TcpListener, Route};
use poem_openapi::{param::Query, payload::Json, ApiResponse, Object, OpenApi, OpenApiService};

use poem::{http::Method, middleware::Cors, EndpointExt};

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

#[derive(Object)]
struct RoomList {
    data: Vec<RoomId>,
}

#[derive(ApiResponse)]
enum GetRoomsResponse {
    #[oai(status = 200)]
    Ok(Json<RoomList>),
    // handled by poem
    // #[oai(status = 400)]
    // Invalid,
    #[oai(status = 401)]
    Unauthorized,
    #[oai(status = 500)]
    Error,
}

struct Api;

#[OpenApi]
impl Api {
    #[oai(path = "/login", method = "post")]
    async fn login(&self, req: Json<LoginRequest>) -> LoginResponse {
        if req.0.username == "admin" && req.0.password == "password" {
            let uid = 0;
            let api_key = gen_token(uid);

            LoginResponse::Ok(Json(LoginResponseBody { api_key }))
        } else {
            LoginResponse::Unauthorized
        }
    }

    #[oai(path = "/rooms", method = "get")]
    async fn rooms(&self, auth: ApiKeyAuthN, pn: Query<Option<u32>>) -> GetRoomsResponse {
        if !check_token(&auth) {
            return GetRoomsResponse::Unauthorized;
        }

        let uid = uid_from_token(&auth);
        let pn = pn.unwrap_or(0) as u64;
        GetRoomsResponse::Ok(Json(RoomList {
            data: vec![pn, uid, 2],
        }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    let mongodb_uri = env::var("MONGODB_URI").expect("MONGODB_URI not found in .env file.");

    let api_service = OpenApiService::new(Api, "RESTful Chat Server in Rust", "0.1")
        .server("http://localhost:3000/chat");
    let ui = api_service.swagger_ui();
    let cors = Cors::new().allow_methods([Method::POST, Method::GET, Method::OPTIONS]);

    let app = Route::new()
        .nest("/chat", api_service)
        .nest("/", ui)
        .with(cors);

    Ok(poem::Server::new(TcpListener::bind("127.0.0.1:3000"))
        .run(app)
        .await?)
}
