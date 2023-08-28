use poem::{listener::TcpListener, Route};
use poem::{web::Data, Request};
use poem_openapi::{param::Query, payload::Json, ApiResponse, Object, OpenApi, OpenApiService};

use poem::{http::Method, middleware::Cors, EndpointExt};

use bson::DateTime;

use hmac::{Hmac, Mac};
use jwt::{SignWithKey, VerifyWithKey};
use poem_openapi::{auth::ApiKey, SecurityScheme};
use serde::{Deserialize, Serialize};
use sha2::Sha256;

type Timestamp = DateTime; // align with mongodb
type UserId = u64;
type RoomId = u64;

const SERVER_KEY: &[u8] = b"secret";

#[derive(Serialize, Deserialize)]
struct JWTPayload {
    // RFC 7519
    sub: UserId,
    exp: Timestamp,
}

#[derive(SecurityScheme)]
#[oai(
    ty = "api_key",
    key_name = "X-API-Key",
    key_in = "header",
    checker = "api_key_checker"
)]
struct ApiAuthN(JWTPayload);

async fn api_key_checker(req: &Request, api_key: ApiKey) -> Option<JWTPayload> {
    let hmac_factory = req.data::<Hmac<Sha256>>().unwrap();
    VerifyWithKey::<JWTPayload>::verify_with_key(api_key.key.as_str(), hmac_factory).ok()
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

#[derive(Object)]
struct RoomList {
    data: Vec<RoomId>
}

#[derive(ApiResponse)]
enum GetRoomsResponse {
    #[oai(status = 200)]
    Ok(Json<RoomList>),
    // handled by poem
    // #[oai(status = 400)]
    // Invalid,
    // #[oai(status = 401)]
    // Unauthorized,
    #[oai(status = 500)]
    Error,
}

struct Api;

#[OpenApi]
impl Api {
    #[oai(path = "/login", method = "post")]
    async fn login(
        &self,
        hmac_factory: Data<&Hmac<Sha256>>,
        req: Json<LoginRequest>,
    ) -> LoginResponse {
        if req.0.username == "admin" && req.0.password == "password" {
            let try_sign = JWTPayload {
                sub: 0,
                exp: DateTime::now(),
            }
            .sign_with_key(hmac_factory.0);

            match try_sign {
                Ok(api_key) => LoginResponse::Ok(Json(LoginResponseBody { api_key })),
                Err(_) => LoginResponse::Error,
            }
        } else {
            LoginResponse::Unauthorized
        }
    }

    #[oai(path = "/rooms", method = "get")]
    async fn rooms(&self, auth: ApiAuthN, pn: Query<Option<u32>>) -> GetRoomsResponse {
        let pn = pn.unwrap_or(0) as u64;
        let uid = auth.0.sub;
        GetRoomsResponse::Ok(Json(RoomList {
            data: vec![pn, uid, 2],
        }))
    }
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let api_service =
        OpenApiService::new(Api, "RESTful Chat Server in Rust", "0.1").server("http://localhost:3000/chat");
    let ui = api_service.swagger_ui();

    let cors = Cors::new().allow_methods([Method::POST, Method::GET, Method::OPTIONS]);
    let hmac_factory =
        Hmac::<Sha256>::new_from_slice(SERVER_KEY).expect("valid server key required");

    let app = Route::new()
        .nest("/chat", api_service)
        .nest("/", ui)
        .with(cors)
        .data(hmac_factory);

    poem::Server::new(TcpListener::bind("127.0.0.1:3000"))
        .run(app)
        .await
}