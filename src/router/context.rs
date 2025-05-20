pub(crate) struct Context {
    pub(crate) postgres: tokio_postgres::Client,
    pub(crate) rabbit: amqprs::channel::Channel,
}

impl Context {
    pub(crate) fn new(postgres: tokio_postgres::Client, rabbit: amqprs::channel::Channel) -> Self {
        Self { postgres, rabbit }
    }
}
