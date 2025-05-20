use std::sync::Arc;

use http_body_util::{Either, Full};
use hyper::{
    Request, Response,
    body::{Bytes, Incoming},
    service::Service,
};

use crate::router::Router;

pub(crate) fn get_user_balance(
    context: Arc<super::super::router::context::Context>,
    request: hyper::Request<Incoming>,
) -> <Router as Service<Request<Incoming>>>::Future {
    Box::pin(async move {
        let user_id: &str = request.headers().get("user_id").unwrap().to_str().unwrap();

        match crate::database::helpers::transaction::get_user_balance(&context.postgres, user_id)
            .await
        {
            Ok(balance) => Response::builder()
                .header("Content-Type", "application/json")
                .status(200)
                .body(Either::Left(Full::from(Bytes::from(format!(
                    "{{\"balance\":{}}}",
                    balance
                ))))),

            Err(_) => crate::utils::generate_error_response(500, "Internal Server Error"),
        }
    })
}
