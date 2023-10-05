use crate::types::*;

use std::env;

use hmac::{Hmac, Mac};
use jwt::{SignWithKey, VerifyWithKey};
use serde::{Deserialize, Serialize};
use sha2::Sha256;

use time::Duration;

use poem::Request;
use poem_openapi::{auth::ApiKey, SecurityScheme};

lazy_static! {
    static ref HMAC_SIGNER: Hmac<Sha256> = {
        // .env already loaded in main()
        let server_key = env::var("SERVER_KEY").expect("SERVER_KEY not found in .env file.");
        Hmac::<Sha256>::new_from_slice(server_key.as_bytes()).expect("valid server key required")
    };
}

const JWT_LIFESPAN: Duration = Duration::minutes(15);

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

async fn api_key_checker(req: &Request, api_key: ApiKey) -> Option<JWTPayload> {
    let hmac_factory = req.data::<Hmac<Sha256>>().unwrap();
    VerifyWithKey::<JWTPayload>::verify_with_key(api_key.key.as_str(), hmac_factory).ok()
}

pub fn gen_token(uid: UserId) -> String {
    let exp_time = Timestamp::now().to_time_0_3() + JWT_LIFESPAN;

    JWTPayload {
        sub: uid,
        exp: Timestamp::from_time_0_3(exp_time),
    }
    .sign_with_key(&(*HMAC_SIGNER))
    .ok()
    .unwrap()
}

// poem handles signature mismatch/badly formed token with 401
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
