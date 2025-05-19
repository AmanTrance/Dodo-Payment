use hyper::{Request, body::Incoming, service::Service};

use crate::router::Router;

pub(crate) fn verify_user(
    request: &mut Request<Incoming>,
) -> Option<<Router as Service<Request<Incoming>>>::Future> {
    if let Some(token) = request.headers().get("Authorization") {
        match token.to_str() {
            Ok(value) => {
                if let Ok(id) = super::verify_jwt(value) {
                    request.headers_mut().insert("user_id", id.parse().unwrap());
                    None
                } else {
                    Some(Box::pin(async move {
                        crate::utils::generate_error_response(401, "Unauthorized")
                    }))
                }
            }

            Err(_) => Some(Box::pin(async move {
                crate::utils::generate_error_response(401, "Unauthorized")
            })),
        }
    } else {
        Some(Box::pin(async move {
            crate::utils::generate_error_response(401, "Unauthorized")
        }))
    }
}
