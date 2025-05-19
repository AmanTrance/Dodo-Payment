pub(crate) mod context;
pub(crate) mod stream;

use http_body_util::{Either, Full};
use hyper::{
    Request, Response,
    body::{Bytes, Incoming},
    service::Service,
};
use std::{pin::Pin, sync::Arc};

pub(crate) struct Router(Arc<context::Context>);

impl Router {
    pub(crate) async fn new() -> Self {
        let result = crate::database::connection::create_postgres_connection().await;
        match result {
            Ok(client) => Router(Arc::new(context::Context::new(client))),
            Err(e) => {
                println!("{:?}", e);
                std::process::exit(1)
            }
        }
    }
}

impl Service<Request<Incoming>> for Router {
    type Error = hyper::http::Error;
    type Response = Response<Either<Full<Bytes>, stream::EventStreamBody>>;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, request: hyper::Request<Incoming>) -> Self::Future {
        match (request.uri().path(), request.method().as_str()) {
            ("/v1/auth/signup", "POST") => {
                crate::handlers::auth::handle_signup(Arc::clone(&self.0), request)
            }

            ("/v1/auth/signin", "POST") => {
                crate::handlers::auth::handle_signin(Arc::clone(&self.0), request)
            }

            ("/v1/profile/get", "GET") => crate::handlers::with_middlewares(
                request,
                Arc::clone(&self.0),
                vec![crate::handlers::middleware::verify_user],
                crate::handlers::profile::handle_get_profile,
            ),

            ("/v1/profile/update", "PUT") => crate::handlers::with_middlewares(
                request,
                Arc::clone(&self.0),
                vec![crate::handlers::middleware::verify_user],
                crate::handlers::profile::handle_profile_update,
            ),

            (_, _) => crate::handlers::not_found(),
        }
    }
}
