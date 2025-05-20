use std::sync::Arc;

use futures_util::TryStreamExt;
use http_body_util::{Either, Full};
use hyper::{
    Request, Response,
    body::{Bytes, Incoming},
    service::Service,
};

use crate::{database::dto::transaction::GetTransactionDTO, router::Router};

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
            SELECT id, tx_time, from_user, to_user, amount, tx_status WHERE user_id = $1
        "#,
                vec![user_id],
            )
            .await
        {
            Ok(rows) => {
                let mut transactions: Vec<GetTransactionDTO> =
                    Vec::with_capacity(rows.rows_affected().unwrap() as usize);
                tokio::pin!(rows);

                while let Ok(Some(row)) = rows.try_next().await {
                    transactions.push(GetTransactionDTO {
                        id: row.get::<&str, i64>("id"),
                        tx_time: row.get::<&str, chrono::NaiveDateTime>("tx_time"),
                        from: row.get::<&str, Option<String>>("from_user"),
                        to: row.get::<&str, Option<String>>("to"),
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

            Err(_) => crate::utils::generate_error_response(500, "Internal Server Error"),
        }
    })
}

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
