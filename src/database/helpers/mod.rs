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
