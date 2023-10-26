use crate::types::*;

use std::sync::OnceLock;

use hmac::{Hmac, Mac};
use jwt::{SignWithKey, VerifyWithKey};
use serde::{Deserialize, Serialize};
use sha2::Sha256;

use time::Duration;

use poem::Request;
use poem_openapi::{auth::ApiKey, SecurityScheme};

const JWT_LIFESPAN: Duration = Duration::minutes(15);

static HMAC_SIGNER: OnceLock<Hmac<Sha256>> = OnceLock::new();

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
pub struct ApiKeyAuthN(JWTPayload);

async fn api_key_checker(_: &Request, api_key: ApiKey) -> Option<JWTPayload> {
    VerifyWithKey::<JWTPayload>::verify_with_key(api_key.key.as_str(), HMAC_SIGNER.get().unwrap())
        .ok()
}

pub fn init_jwt(server_key: String) {
    HMAC_SIGNER.get_or_init(|| {
        Hmac::<Sha256>::new_from_slice(server_key.as_bytes()).expect("valid server key required")
    });
}

pub fn gen_token(uid: UserId) -> String {
    let exp_time = Timestamp::now().to_time_0_3() + JWT_LIFESPAN;

    JWTPayload {
        sub: uid,
        exp: Timestamp::from_time_0_3(exp_time),
    }
    .sign_with_key(HMAC_SIGNER.get().unwrap())
    .ok()
    .unwrap()
}

// poem handles signature mismatch/badly formed token with 400
// this function takes a poem wrapper of a well formed and checked JWT object
// then performs user-defined checks, returns true on passing
pub fn check_token(auth: &ApiKeyAuthN) -> bool {
    let jwt = &auth.0;

    Timestamp::now() < jwt.exp
}

pub fn uid_from_token(auth: &ApiKeyAuthN) -> UserId {
    let jwt = &auth.0;

    jwt.sub
}
