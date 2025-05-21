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
    database::dto::transaction::GetTransactionDTO, events::transaction::TransactionHandlerDTO,
    router::Router,
};

pub(crate) fn create_transaction(
    context: Arc<super::super::router::context::Context>,
    mut request: hyper::Request<Incoming>,
) -> <Router as Service<Request<Incoming>>>::Future {
    Box::pin(async move {
        let body: Result<Collected<Bytes>, hyper::Error> = request.body_mut().collect().await;
        let user_id: &str = request.headers().get("user_id").unwrap().to_str().unwrap();

        match body {
            Ok(bytes) => {
                if let Ok(request_dto) = serde_json::from_slice::<
                    crate::database::dto::transaction::CreateTransactionDTO,
                >(bytes.to_bytes().as_ref())
                {
                    match context
                        .postgres
                        .query_one(
                            r#"
                        SELECT created_by FROM upis WHERE upi_id = $1
                    "#,
                            &[&request_dto.to],
                        )
                        .await
                    {
                        Ok(row) => {
                            let query_user_id: String = row.get::<&str, String>("created_by");
                            let transaction_dto: TransactionHandlerDTO = TransactionHandlerDTO {
                                from: String::from(user_id),
                                to: String::from(query_user_id),
                                amount: request_dto.amount.unwrap(),
                                is_external: false,
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
                        }

                        Err(_) => {
                            crate::utils::generate_error_response(400, "Sender UPI Not Exist")
                        }
                    }
                } else {
                    crate::utils::generate_error_response(400, "Bad Request")
                }
            }

            Err(_) => crate::utils::generate_error_response(500, "Internal Server Error"),
        }
    })
}

pub(crate) fn get_transactions_list(
    context: Arc<super::super::router::context::Context>,
    request: hyper::Request<Incoming>,
) -> <Router as Service<Request<Incoming>>>::Future {
    Box::pin(async move {
        let user_id: &str = request.headers().get("user_id").unwrap().to_str().unwrap();

        match context
            .postgres
            .query_raw::<str, &str, Vec<&str>>(
                r#"
            SELECT id, tx_time, from_user, to_user, amount::float, tx_status, is_external FROM transactions WHERE user_id = $1 ORDER BY tx_time DESC
        "#,
                vec![user_id],
            )
            .await
        {
            Ok(rows) => {
                let mut transactions: Vec<GetTransactionDTO> = Vec::new();
                tokio::pin!(rows);

                while let Ok(Some(row)) = rows.try_next().await {
                    transactions.push(GetTransactionDTO {
                        id: row.get::<&str, i32>("id"),
                        tx_time: row.get::<&str, chrono::NaiveDateTime>("tx_time"),
                        from: row.get::<&str, Option<String>>("from_user"),
                        to: row.get::<&str, Option<String>>("to_user"),
                        amount: row.get::<&str, f64>("amount"),
                        tx_status: row.get::<&str, String>("tx_status"),
                        is_external: row.get::<&str, bool>("is_external"),
                    });
                }

                Response::builder()
                    .header("Content-Type", "application/json")
                    .status(200)
                    .body(Either::Left(Full::from(Bytes::from(
                        serde_json::to_vec(&transactions).unwrap(),
                    ))))
            }

            Err(_) => crate::utils::generate_error_response(500, "Internal Server Error")

        }
    })
}

pub(crate) fn get_user_balance(
    context: Arc<super::super::router::context::Context>,
    request: hyper::Request<Incoming>,
) -> <Router as Service<Request<Incoming>>>::Future {
    Box::pin(async move {
        let user_id: &str = request.headers().get("user_id").unwrap().to_str().unwrap();

        let balance: f64 =
            crate::database::helpers::transaction::get_user_balance(&context.postgres, user_id)
                .await;
        Response::builder()
            .header("Content-Type", "application/json")
            .status(200)
            .body(Either::Left(Full::from(Bytes::from(format!(
                "{{\"balance\":{}}}",
                balance
            )))))
    })
}
