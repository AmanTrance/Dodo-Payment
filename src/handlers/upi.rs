use std::sync::Arc;

use amqprs::{BasicProperties, channel::BasicPublishArguments};
use futures_util::TryStreamExt;
use http_body_util::{BodyExt, Collected, Either, Full};
use hyper::{
    Request, Response,
    body::{Bytes, Incoming},
    service::Service,
};

use crate::{
    database::dto::upi::UpiGetDTO, events::transaction::TransactionHandlerDTO, router::Router,
};

pub(crate) fn get_upis(
    context: Arc<crate::router::context::Context>,
    request: Request<Incoming>,
) -> <Router as Service<Request<Incoming>>>::Future {
    Box::pin(async move {
        let user_id: &str = request.headers().get("user_id").unwrap().to_str().unwrap();

        match context
            .postgres
            .query_raw::<str, &str, Vec<&str>>(
                r#"
            SELECT created_at, upi_id, is_default FROM upis WHERE created_by = $1
        "#,
                vec![user_id],
            )
            .await
        {
            Ok(rows) => {
                let mut upis: Vec<UpiGetDTO> = Vec::new();
                tokio::pin!(rows);

                while let Ok(Some(row)) = rows.try_next().await {
                    upis.push(UpiGetDTO {
                        created_at: row.get::<&str, chrono::NaiveDateTime>("created_at"),
                        upi_id: row.get::<&str, String>("upi_id"),
                        is_default: row.get::<&str, bool>("is_default"),
                    });
                }

                Response::builder()
                    .header("Content-Type", "application/json")
                    .status(200)
                    .body(Either::Left(Full::from(Bytes::from(
                        serde_json::to_vec(&upis).unwrap(),
                    ))))
            }

            Err(_) => crate::utils::generate_error_response(500, "Internal Server Error"),
        }
    })
}

pub(crate) fn fund_upi(
    context: Arc<crate::router::context::Context>,
    mut request: Request<Incoming>,
) -> <Router as Service<Request<Incoming>>>::Future {
    Box::pin(async move {
        let body: Result<Collected<Bytes>, hyper::Error> = request.body_mut().collect().await;
        let user_id: &str = request.headers().get("user_id").unwrap().to_str().unwrap();

        match body {
            Ok(bytes) => {
                if let Ok(user) = serde_json::from_slice::<crate::database::dto::upi::FundUpiDTO>(
                    bytes.to_bytes().as_ref(),
                ) {
                    match context
                        .postgres
                        .query_one(
                            r#"
                        SELECT created_by FROM upis WHERE upi_id = $1
                    "#,
                            &[&user.upi_id.unwrap()],
                        )
                        .await
                    {
                        Ok(row) => {
                            let query_user_id: String = row.get::<&str, String>("created_by");
                            if user_id[..] == query_user_id[..] {
                                let transaction_dto: TransactionHandlerDTO =
                                    TransactionHandlerDTO {
                                        from: String::from(user_id),
                                        to: String::from(user_id),
                                        amount: user.amount.unwrap(),
                                        is_external: true,
                                    };

                                let _ = context
                                    .rabbitmq
                                    .basic_publish(
                                        BasicProperties::default()
                                            .with_content_type("application/json")
                                            .with_content_encoding("utf-8")
                                            .finish(),
                                        serde_json::to_vec(&transaction_dto).unwrap(),
                                        BasicPublishArguments::default()
                                            .exchange("amq.direct".into())
                                            .routing_key("transactions".into())
                                            .finish(),
                                    )
                                    .await;

                                Response::builder()
                                    .header("Content-Type", "application/json")
                                    .status(200)
                                    .body(Either::Left(Full::from(Bytes::from(format!(
                                        "{{\"message\":\"Your Transaction will be Processed Shortly\"}}"
                            )))))
                            } else {
                                crate::utils::generate_error_response(401, "Unauthorized Access")
                            }
                        }

                        Err(_) => crate::utils::generate_error_response(400, "UPI Not Found"),
                    }
                } else {
                    crate::utils::generate_error_response(400, "Bad Request")
                }
            }

            Err(_) => crate::utils::generate_error_response(400, "Bad Request"),
        }
    })
}
