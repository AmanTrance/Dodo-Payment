use std::sync::Arc;

use http_body_util::{BodyExt, Collected, Either, Full};
use hyper::{
    Request, Response,
    body::{Bytes, Incoming},
    service::Service,
};

use crate::{
    database::dto::user::{UserProfileGetDTO, UserProfileUpdateDTO},
    router::Router,
};

pub(crate) fn handle_profile_update(
    context: Arc<super::super::router::context::Context>,
    mut request: hyper::Request<Incoming>,
) -> <Router as Service<Request<Incoming>>>::Future {
    Box::pin(async move {
        let body: Result<Collected<Bytes>, hyper::Error> = request.body_mut().collect().await;

        match body {
            Ok(bytes) => {
                if let Ok(profile_update_dto) =
                    serde_json::from_slice::<UserProfileUpdateDTO>(bytes.to_bytes().as_ref())
                {
                    if profile_update_dto.city.is_none()
                        && profile_update_dto.state.is_none()
                        && profile_update_dto.country.is_none()
                        && profile_update_dto.avatar.is_none()
                    {
                        crate::utils::generate_error_response(400, "Nothing to Update")
                    } else {
                        let user_id: &str =
                            request.headers().get("user_id").unwrap().to_str().unwrap();
                        let mut update_query: String = String::from("UPDATE users SET");

                        if !profile_update_dto.city.is_none() {
                            update_query.push_str(
                                format!(" city = '{}',", profile_update_dto.city.unwrap()).as_str(),
                            );
                        }

                        if !profile_update_dto.state.is_none() {
                            update_query.push_str(
                                format!(" state = '{}',", profile_update_dto.state.unwrap())
                                    .as_str(),
                            );
                        }

                        if !profile_update_dto.country.is_none() {
                            update_query.push_str(
                                format!(" country = '{}',", profile_update_dto.country.unwrap())
                                    .as_str(),
                            );
                        }

                        if !profile_update_dto.avatar.is_none() {
                            update_query.push_str(
                                format!(" avatar = '{}',", profile_update_dto.avatar.unwrap())
                                    .as_str(),
                            );
                        }

                        update_query.pop();
                        update_query.push_str(format!(" WHERE id = '{}'", user_id).as_str());

                        println!("{}", update_query);

                        match context
                            .postgres
                            .execute_raw::<str, String, Vec<String>>(&update_query, vec![])
                            .await
                        {
                            Ok(_) => Response::builder()
                                .header("Content-Type", "application/json")
                                .status(200)
                                .body(Either::Left(Full::from(Bytes::from(
                                    "{\"message\":\"Profile Updated Successfully\"}",
                                )))),

                            Err(e) => {
                                println!("{}", e.to_string());
                                crate::utils::generate_error_response(500, "Internal Server Error")
                            }
                        }
                    }
                } else {
                    crate::utils::generate_error_response(400, "Bad Request")
                }
            }

            Err(_) => crate::utils::generate_error_response(400, "Bad Request"),
        }
    })
}

pub(crate) fn handle_get_profile(
    context: Arc<super::super::router::context::Context>,
    request: hyper::Request<Incoming>,
) -> <Router as Service<Request<Incoming>>>::Future {
    Box::pin(async move {
        let user_id: &str = request.headers().get("user_id").unwrap().to_str().unwrap();

        match context
            .postgres
            .query_one(
                r#"
            SELECT created_at, username, email, city, state, country, avatar FROM users WHERE id = $1
        "#,
                &[&user_id],
            )
            .await
        {
            Ok(user) => {
                let user_profile_dto: UserProfileGetDTO = UserProfileGetDTO {
                    created_at: user.get::<&str, Option<chrono::NaiveDateTime>>("created_at"),
                    username: user.get::<&str, Option<String>>("username"),
                    email: user.get::<&str, Option<String>>("email"),
                    city: user.get::<&str, Option<String>>("city"),
                    state: user.get::<&str, Option<String>>("state"),
                    country: user.get::<&str, Option<String>>("country"),
                    avatar: user.get::<&str, Option<String>>("avatar"),
                };

                Response::builder()
                    .header("Content-Type", "application/json")
                    .status(200)
                    .body(Either::Left(Full::from(Bytes::from_owner(
                        serde_json::to_string(&user_profile_dto).unwrap(),
                    ))))
            }

            Err(_) => crate::utils::generate_error_response(500, "Internal Server Error"),
        }
    })
}
