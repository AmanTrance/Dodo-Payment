pub(crate) mod context;
pub(crate) mod stream;

use http_body_util::{Either, Full};
use hyper::{
    Request, Response,
    body::{Bytes, Incoming},
    service::Service,
};
use std::{pin::Pin, sync::Arc};

use crate::EventHandlerDTO;

pub(crate) struct Router(Arc<context::Context>);

impl Router {
    pub(crate) async fn new(
        postgres: tokio_postgres::Client,
        rabbit: amqprs::channel::Channel,
        register: tokio::sync::mpsc::UnboundedSender<EventHandlerDTO>,
    ) -> Self {
        Router(Arc::new(context::Context::new(postgres, rabbit, register)))
    }
}

impl Service<Request<Incoming>> for Router {
    type Error = hyper::http::Error;
    type Response = Response<Either<Full<Bytes>, stream::EventStreamBody>>;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    #[inline(always)]
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

            ("/v1/upi/list", "GET") => crate::handlers::with_middlewares(
                request,
                Arc::clone(&self.0),
                vec![crate::handlers::middleware::verify_user],
                crate::handlers::upi::get_upis,
            ),

            ("/v1/upi/fund", "POST") => crate::handlers::with_middlewares(
                request,
                Arc::clone(&self.0),
                vec![crate::handlers::middleware::verify_user],
                crate::handlers::upi::fund_upi,
            ),

            ("/v1/transaction/list", "GET") => crate::handlers::with_middlewares(
                request,
                Arc::clone(&self.0),
                vec![crate::handlers::middleware::verify_user],
                crate::handlers::transaction::get_transactions_list,
            ),

            // ("/v1/account/balance", "GET") => crate::handlers::with_middlewares(
            //     request,
            //     Arc::clone(&self.0),
            //     vec![crate::handlers::middleware::verify_user],
            //     crate::handlers::transaction::get_user_balance,
            // ),
            ("/v1/events", "GET") => crate::handlers::with_middlewares(
                request,
                Arc::clone(&self.0),
                vec![crate::handlers::middleware::verify_user],
                crate::handlers::events::get_events,
            ),

            (_, _) => crate::handlers::not_found(),
        }
    }
}
