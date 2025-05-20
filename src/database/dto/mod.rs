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

pub(crate) mod upi {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize)]
    pub(crate) struct UpiGetDTO {
        pub(crate) created_at: chrono::NaiveDateTime,
        pub(crate) upi_id: String,
        pub(crate) is_default: bool,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub(crate) struct FundUpiDTO {
        pub(crate) upi_id: Option<String>,
        pub(crate) amount: Option<f64>,
    }
}

pub(crate) mod transaction {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    pub(crate) struct CreateTransactionDTO {
        pub(crate) to: Option<String>,
        pub(crate) amount: Option<f64>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub(crate) struct GetTransactionDTO {
        pub(crate) id: i64,
        pub(crate) tx_time: chrono::NaiveDateTime,
        pub(crate) from: Option<String>,
        pub(crate) to: Option<String>,
        pub(crate) amount: f64,
        pub(crate) is_external: bool,
        pub(crate) tx_status: String,
    }
}
