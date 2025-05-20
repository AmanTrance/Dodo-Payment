pub mod auth;
pub mod events;
pub mod middleware;
pub mod profile;
pub mod transaction;
pub mod upi;

use hmac::{Hmac, Mac, digest::InvalidLength};
use http_body_util::{Either, Full};
use hyper::{
    Request, Response,
    body::{Bytes, Incoming},
    service::Service,
};
use jwt::{Header, SignWithKey, Token, VerifyWithKey, token};
use sha2::Sha384;
use std::{collections::BTreeMap, sync::Arc};

use crate::router::{Router, context::Context};

pub fn not_found() -> <Router as Service<Request<Incoming>>>::Future {
    Box::pin(async move {
        Response::builder()
            .header("Content-Type", "application/json")
            .status(200)
            .body(Either::Left(Full::new(Bytes::from(
                "{\"error\":\"Page Not Found\"}",
            ))))
    })
}

pub fn generate_jwt(user_id: &str) -> Result<String, InvalidLength> {
    let key: Hmac<Sha384> = Hmac::new_from_slice(b"some-secret")?;
    let header: Header = Header {
        algorithm: jwt::AlgorithmType::Hs384,
        ..Default::default()
    };
    let mut claims: BTreeMap<&str, &str> = BTreeMap::<&str, &str>::new();
    claims.insert("user_id", user_id);
    let token: Token<Header, BTreeMap<&str, &str>, token::Signed> =
        Token::new(header, claims).sign_with_key(&key).unwrap();

    Ok(token.into())
}

pub fn verify_jwt(token_string: &str) -> Result<String, InvalidLength> {
    let key: Hmac<Sha384> = Hmac::new_from_slice(b"some-secret")?;
    let token: Token<Header, BTreeMap<String, String>, _> =
        token_string.verify_with_key(&key).unwrap();
    let claims: &BTreeMap<String, String> = token.claims();

    Ok(claims.get("user_id").unwrap().to_owned())
}

pub fn with_middlewares(
    mut request: Request<Incoming>,
    context: Arc<Context>,
    middlewares: Vec<
        fn(&mut Request<Incoming>) -> Option<<Router as Service<Request<Incoming>>>::Future>,
    >,
    func: fn(Arc<Context>, Request<Incoming>) -> <Router as Service<Request<Incoming>>>::Future,
) -> <Router as Service<Request<Incoming>>>::Future {
    for f in middlewares {
        if let Some(response) = f(&mut request) {
            return response;
        } else {
            continue;
        }
    }

    func(context, request)
}
