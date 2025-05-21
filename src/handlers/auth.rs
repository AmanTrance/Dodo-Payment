use http_body_util::{BodyExt, Collected, Either, Full};
use hyper::{
    Request, Response,
    body::{Bytes, Incoming},
    service::Service,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::database::dto::user::UserCreateDTO;
use crate::{database::dto::user::UserSignupDTO, router::Router};

pub(crate) fn handle_signup(
    context: Arc<super::super::router::context::Context>,
    request: hyper::Request<Incoming>,
) -> <Router as Service<Request<Incoming>>>::Future {
    Box::pin(async move {
        let body: Result<Collected<Bytes>, hyper::Error> = request.into_body().collect().await;

        match body {
            Ok(bytes) => {
                if let Ok(mut user) =
                    serde_json::from_slice::<UserCreateDTO>(bytes.to_bytes().as_ref())
                {
                    if user.username.is_none() || user.email.is_none() || user.password.is_none() {
                        crate::utils::generate_error_response(400, "Bad Payload")
                    } else {
                        user.password = Some(
                            bcrypt::hash(user.password.take().unwrap(), bcrypt::DEFAULT_COST)
                                .unwrap(),
                        );
                        user.id = Some(Uuid::new_v4().to_string());

                        match context
                            .postgres
                            .execute_raw(
                                r#"
                            INSERT INTO users (id, username, email, password) VALUES ($1, $2, $3, $4)
                        "#,
                                vec![
                                    user.id.as_ref().unwrap().clone(),
                                    user.username.as_ref().unwrap().clone(),
                                    user.email.unwrap(),
                                    user.password.unwrap(),
                                ],
                            )
                            .await
                        {
                            Ok(_) => match super::generate_jwt(user.id.as_ref().unwrap()) {
                                Ok(token) => {
                                    let user_upi: String = format!("{}@dodo", user.username.unwrap());
                                    match context.postgres.execute_raw(r#"
                                        INSERT INTO upis (upi_id, is_default, created_by) VALUES ($1, TRUE, $2)
                                    "#, vec![
                                        user_upi.clone(),
                                        user.id.unwrap()
                                    ]).await {
                                        Ok(_) => {
                                            Response::builder()
                                                .header("Content-Type", "application/json")
                                                .status(200)
                                                .body(Either::Left(Full::from(Bytes::from(format!(
                                                    "{{\"token\":\"{}\", \"upi_id\":\"{}\"}}"
                                            ,token, user_upi)))))
                                        }

                                        Err(_) => crate::utils::generate_error_response(
                                            500,
                                            "Internal Server Error",
                                        ),
                                    }
                                }

                                Err(_) => crate::utils::generate_error_response(
                                    500,
                                    "Internal Server Error",
                                ),
                            },

                            Err(_) => {
                                crate::utils::generate_error_response(400, "User Already Exists With Either Email Or Username")
                            }
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

pub(crate) fn handle_signin(
    context: Arc<super::super::router::context::Context>,
    request: hyper::Request<Incoming>,
) -> <Router as Service<Request<Incoming>>>::Future {
    Box::pin(async move {
        let body: Result<Collected<Bytes>, hyper::Error> = request.into_body().collect().await;

        match body {
            Ok(bytes) => {
                if let Ok(signup_dto) =
                    serde_json::from_slice::<UserSignupDTO>(bytes.to_bytes().as_ref())
                {
                    if signup_dto.email.is_none() || signup_dto.password.is_none() {
                        crate::utils::generate_error_response(400, "Bad Payload")
                    } else {
                        match context
                            .postgres
                            .query_one(
                                r#"
                            SELECT id, password FROM users WHERE email = $1
                        "#,
                                &[&signup_dto.email.unwrap()],
                            )
                            .await
                        {
                            Ok(row) => {
                                let id: &str = row.get::<&str, &str>("id");
                                let password: &str = row.get::<&str, &str>("password");

                                if bcrypt::verify(signup_dto.password.unwrap(), password).unwrap() {
                                    match super::generate_jwt(id) {
                                        Ok(token) => Response::builder()
                                            .header("Content-Type", "application/json")
                                            .status(200)
                                            .body(Either::Left(Full::from(format!(
                                                "{{\"token\":\"{}\"}}",
                                                token
                                            )))),

                                        Err(_) => crate::utils::generate_error_response(
                                            500,
                                            "Internal Server Error",
                                        ),
                                    }
                                } else {
                                    crate::utils::generate_error_response(401, "Unauthorized")
                                }
                            }

                            Err(_) => crate::utils::generate_error_response(400, "User Not Found"),
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
