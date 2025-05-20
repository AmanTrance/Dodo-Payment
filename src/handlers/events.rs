use std::sync::Arc;

use http_body_util::Either;
use hyper::{
    Request, Response,
    body::{Bytes, Incoming},
    service::Service,
};

use crate::router::Router;

pub(crate) fn get_events(
    context: Arc<crate::router::context::Context>,
    request: Request<Incoming>,
) -> <Router as Service<Request<Incoming>>>::Future {
    Box::pin(async move {
        let user_id: &str = request.headers().get("user_id").unwrap().to_str().unwrap();

        let (sender, receiver) = tokio::sync::mpsc::channel::<Bytes>(10);

        let _ = context
            .register
            .send(crate::events::EventHandlerDTO::RegisterSender {
                user_id: user_id.into(),
                sender,
            });

        Response::builder()
            .header("Content-Type", "text/event-stream")
            .header("Cache-Control", "no-cache")
            .header("Connection", "keep-alive")
            .status(200)
            .body(Either::Right(crate::router::stream::EventStreamBody::new(
                receiver,
            )))
    })
}
