pub(crate) struct Context {
    pub(crate) postgres: tokio_postgres::Client,
}

impl Context {
    pub fn new(postgres: tokio_postgres::Client) -> Self {
        Self { postgres }
    }
}
