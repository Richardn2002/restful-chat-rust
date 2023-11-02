use crate::jwt::{check_token, uid_from_token, ApiKeyAuthN};
use crate::types::*;

use poem_openapi::{param::Query, payload::Json, ApiResponse, Object, OpenApi};

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

pub struct Chat;

#[OpenApi]
impl Chat {
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
