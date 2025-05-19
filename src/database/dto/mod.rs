pub(crate) mod user {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    pub(crate) struct UserCreateDTO {
        #[serde(skip_deserializing)]
        pub(crate) id: Option<String>,
        pub(crate) username: Option<String>,
        pub(crate) email: Option<String>,
        pub(crate) password: Option<String>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub(crate) struct UserSignupDTO {
        pub(crate) email: Option<String>,
        pub(crate) password: Option<String>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub(crate) struct UserProfileUpdateDTO {
        pub(crate) city: Option<String>,
        pub(crate) state: Option<String>,
        pub(crate) country: Option<String>,
        pub(crate) avatar: Option<String>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub(crate) struct UserProfileGetDTO {
        pub(crate) created_at: Option<chrono::NaiveDateTime>,
        pub(crate) username: Option<String>,
        pub(crate) email: Option<String>,
        pub(crate) city: Option<String>,
        pub(crate) state: Option<String>,
        pub(crate) country: Option<String>,
        pub(crate) avatar: Option<String>,
    }
}
