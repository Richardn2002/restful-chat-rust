pub mod auth;
pub mod jwt;
pub mod types;

use crate::auth::Auth;
use crate::jwt::{check_token, init_jwt, uid_from_token, ApiKeyAuthN};
use crate::types::*;

use dotenv::dotenv;
use std::env;

use std::error::Error;

use poem::{listener::TcpListener, Route};
use poem_openapi::{param::Query, payload::Json, ApiResponse, Object, OpenApi, OpenApiService};

use poem::{http::Method, middleware::Cors, EndpointExt};

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
    let server_key = env::var("SERVER_KEY").expect("SERVER_KEY not found in .env file.");
    init_jwt(server_key);

    let api_service = OpenApiService::new(Api, "RESTful Chat Server in Rust", "0.1")
        .server("http://localhost:3000/chat");
    let ui = api_service.swagger_ui();

    let auth_service = OpenApiService::new(Auth, "RESTful Chat Server in Rust", "0.1")
        .server("http://localhost:3000/auth");
    let auth_ui = auth_service.swagger_ui();

    let cors = Cors::new().allow_methods([Method::POST, Method::GET, Method::OPTIONS]);

    let app = Route::new()
        .nest("/chat", api_service)
        .nest("/chat/ui", ui)
        .nest("/auth", auth_service)
        .nest("/auth/ui", auth_ui)
        .with(cors);

    Ok(poem::Server::new(TcpListener::bind("127.0.0.1:3000"))
        .run(app)
        .await?)
}
