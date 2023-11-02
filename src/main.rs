mod auth;
mod chat;
mod db;
mod jwt;
mod types;

use crate::auth::Auth;
use crate::chat::Chat;
use crate::db::Db;
use crate::jwt::init_jwt;

use dotenv::dotenv;
use std::env;

use std::error::Error;

use poem::{listener::TcpListener, Route};
use poem_openapi::OpenApiService;

use poem::{http::Method, middleware::Cors, EndpointExt};

fn get_from_env(key: &str) -> String {
    env::var(key).unwrap_or_else(|_| panic!("{key} not found in .env file."))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let server_key = get_from_env("SERVER_KEY");
    init_jwt(server_key);

    let mongodb_uri = get_from_env("MONGODB_URI");
    let auth_db_name = get_from_env("AUTH_DB_NAME");
    let creds_coll_name = get_from_env("CREDS_COLL_NAME");
    let db = Db::new(mongodb_uri)
        .await?
        .init_auth(auth_db_name, creds_coll_name)
        .await?;

    // db.insert_creds(
    //     "admin",
    //     "$2a$12$f7h31gSiD0e22vCLJss6ROst5kTYD3G0MIzyzpO9ef.FQ8W4Px3ee",
    //     0,
    // )
    // .await
    // .map_err(Box::new)?;

    let api_service =
        OpenApiService::new(Chat, "RESTful Chat Server in Rust - Chat subservice", "0.1")
            .server("http://localhost:3000/chat");
    let ui = api_service.swagger_ui();

    let auth_service =
        OpenApiService::new(Auth, "RESTful Chat Server in Rust - Auth subservice", "0.1")
            .server("http://localhost:3000/auth");
    let auth_ui = auth_service.swagger_ui();

    let cors = Cors::new().allow_methods([Method::POST, Method::GET, Method::OPTIONS]);

    let app = Route::new()
        .nest("/chat", api_service)
        .nest("/chat/ui", ui)
        .nest("/auth", auth_service)
        .nest("/auth/ui", auth_ui)
        .data(db)
        .with(cors);

    Ok(poem::Server::new(TcpListener::bind("127.0.0.1:3000"))
        .run(app)
        .await?)
}
