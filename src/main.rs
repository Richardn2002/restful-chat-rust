use poem::{listener::TcpListener, Route, middleware::Cors, http::Method, EndpointExt};
use poem_openapi::{param::Query, payload::Json, OpenApi, OpenApiService, ApiResponse, Object};

type UserId = u64;
type RoomId = u64;

#[derive(Object)]
struct RoomList {
    data: Vec<RoomId>
}

#[derive(ApiResponse)]
enum GetRoomsResponse {
    #[oai(status = 200)]
    Ok(Json<RoomList>),
    #[oai(status = 400)]
    Invalid,
    #[oai(status = 401)]
    Unauthorized,
    #[oai(status = 500)]
    Error
}

struct Api;

#[OpenApi]
impl Api {
    #[oai(path = "/rooms", method = "get")]
    async fn index(&self, _pn: Query<Option<u32>>) -> GetRoomsResponse {
        GetRoomsResponse::Ok(Json(RoomList { data: vec![_pn.unwrap_or(0) as u64,1,2] }))
    }
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let api_service =
        OpenApiService::new(Api, "RESTful Chat Server in Rust", "0.1").server("http://localhost:3000/chat");
    let ui = api_service.swagger_ui();
    let cors =
    Cors::new()
        .allow_methods([Method::POST, Method::GET, Method::OPTIONS]);
    let app = Route::new().nest("/chat", api_service).nest("/", ui).with(cors);

    poem::Server::new(TcpListener::bind("127.0.0.1:3000"))
        .run(app)
        .await
}