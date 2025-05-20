use std::sync::Arc;

use amqprs::channel::{BasicConsumeArguments, Channel};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct TransactionHandlerDTO {
    pub(crate) from: String,
    pub(crate) to: String,
    pub(crate) amount: f64,
    pub(crate) is_external: bool,
}

pub(crate) async fn setup_transaction_handler(
    postgres: Arc<tokio_postgres::Client>,
    rabbitmq: Channel,
) -> () {
    let consumer_tag: String = uuid::Uuid::new_v4().to_string();

    let (_, mut rabbitmq_recv) = match rabbitmq
        .basic_consume_rx(
            BasicConsumeArguments::default()
                .auto_ack(true)
                .consumer_tag(consumer_tag)
                .finish(),
        )
        .await
    {
        Ok(result) => result,
        Err(_) => std::process::exit(0),
    };

    let value = rabbitmq_recv.recv().await;

    match value {
        Some(message) => match message.content {
            Some(buffer) => {
                let transaction_dto: TransactionHandlerDTO =
                    serde_json::from_slice(&buffer).unwrap();
            }

            None => (),
        },

        None => (),
    }
}
