use std::sync::Arc;

use futures_util::TryStreamExt;
use http_body_util::{Either, Full};
use hyper::{
    Request, Response,
    body::{Bytes, Incoming},
    service::Service,
};

use crate::{database::dto::upi::UpiGetDTO, router::Router};

pub(crate) fn create_upi(
    context: Arc<crate::router::context::Context>,
    request: Request<Incoming>,
) -> <Router as Service<Request<Incoming>>>::Future {
    Box::pin(async move {
        let user_id: &str = request.headers().get("user_id").unwrap().to_str().unwrap();

        match crate::database::helpers::user::get_user_by_id(&context.postgres, user_id).await {
            Ok(_) => {
                match context
                    .postgres
                    .execute_raw::<str, &str, Vec<&str>>(
                        r#"
                    INSERT INTO upis (upi_id, created_by) VALUES ($1, $2)
                "#,
                        vec![user_id],
                    )
                    .await
                {
                    Ok(_) => Response::builder()
                        .header("Content-Type", "application/json")
                        .status(200)
                        .body(Either::Left(Full::from(Bytes::from("")))),

                    Err(_) => crate::utils::generate_error_response(500, "Internal Server Error"),
                }
            }

            Err(_) => crate::utils::generate_error_response(500, "Internal Server Error"),
        }
    })
}

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
            SELECT upi_id FROM upis WHERE created_by = $1
        "#,
                vec![user_id],
            )
            .await
        {
            Ok(rows) => {
                let mut upis: Vec<UpiGetDTO> = vec![];
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

            Err(e) => {
                println!("{}", e.to_string());
                crate::utils::generate_error_response(500, "Internal Server Error")
            }
        }
    })
}
