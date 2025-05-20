use std::sync::Arc;

use amqprs::{channel::{BasicConsumeArguments, BasicPublishArguments, Channel}, BasicProperties};
use hyper::body::Bytes;
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

                let user_balance: f64 = crate::database::helpers::transaction::get_user_balance(postgres.as_ref(), &transaction_dto.from).await.unwrap();
                if user_balance < transaction_dto.amount {
                    let sse_event_dto: super::ChannelDTO = super::ChannelDTO {
                        user_id: transaction_dto.from.clone(),
                        event_name: "Failure".into(),
                        event_data: Bytes::from(serde_json::to_string(&transaction_dto).unwrap()).to_vec()
                    };

                    let _ = rabbitmq.basic_publish(BasicProperties::default(), serde_json::to_vec(&sse_event_dto).unwrap(), BasicPublishArguments::default()).await;
                } else {
                    
                }
            }

            None => (),
        },

        None => (),
    }
}
