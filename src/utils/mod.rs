use http_body_util::{Either, Full};
use hyper::{Response, body::Bytes};

use crate::router::stream::EventStreamBody;

pub fn generate_error_response(
    status_code: u16,
    message: &str,
) -> Result<Response<Either<Full<Bytes>, EventStreamBody>>, hyper::http::Error> {
    Response::builder()
        .header("Content-Type", "application/json")
        .status(status_code)
        .body(Either::Left(Full::from(format!(
            "{{\"error\":\"{}\"}}",
            message
        ))))
}
