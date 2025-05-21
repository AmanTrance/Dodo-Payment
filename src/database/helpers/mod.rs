pub(crate) mod user {
    use crate::database::dto::user::UserProfileGetDTO;

    pub(crate) async fn get_user_by_id(
        client: &tokio_postgres::Client,
        user_id: &str,
    ) -> Result<UserProfileGetDTO, tokio_postgres::Error> {
        let row: tokio_postgres::Row = client.query_one(r#"
            SELECT created_at, username, email, city, state, country, avatar FROM users WHERE id = $1
        "#, &[&user_id]).await?;
        let user: UserProfileGetDTO = UserProfileGetDTO {
            created_at: row.get::<&str, Option<chrono::NaiveDateTime>>("created_at"),
            username: row.get::<&str, Option<String>>("username"),
            email: row.get::<&str, Option<String>>("email"),
            city: row.get::<&str, Option<String>>("city"),
            state: row.get::<&str, Option<String>>("state"),
            country: row.get::<&str, Option<String>>("country"),
            avatar: row.get::<&str, Option<String>>("avatar"),
        };

        Ok(user)
    }
}

pub(crate) mod transaction {

    pub(crate) async fn get_user_balance(client: &tokio_postgres::Client, user_id: &str) -> f64 {
        match client.query_one(r#"
            SELECT SUM(
                CASE 
                    WHEN (from_user IS NULL AND to_user IS NOT NULL AND is_external = FALSE) THEN (-1) * amount
                    WHEN (from_user IS NOT NULL AND to_user IS NULL AND is_external = FALSE) THEN amount
                    ELSE amount
                END
            )::float AS balance FROM transactions WHERE user_id = $1 AND (tx_status = 'SUCCESS' OR tx_status = 'RECEIVED') GROUP BY user_id
        "#, &[&user_id]).await {
            Ok(row) => {
                row.get::<&str, f64>("balance")
            }

            Err(_) => {
                0.0
            }
        }
    }
}
