use std::io::Write;

use amqprs::{
    BasicProperties,
    channel::{BasicConsumeArguments, BasicPublishArguments, Channel},
};
use hyper::body::Bytes;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::Receiver;

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct TransactionHandlerDTO {
    pub(crate) from: String,
    pub(crate) to: String,
    pub(crate) amount: f64,
    pub(crate) is_external: bool,
}

pub(crate) async fn setup_transaction_handler(
    mut oneshot_receiver: Receiver<u8>,
    postgres: tokio_postgres::Client,
    rabbitmq: Channel,
    queue_name: String,
) -> () {
    let consumer_tag: String = uuid::Uuid::new_v4().to_string();

    let (_, mut rabbitmq_recv) = match rabbitmq
        .basic_consume_rx(
            BasicConsumeArguments::default()
                .queue(queue_name)
                .auto_ack(true)
                .consumer_tag(consumer_tag)
                .finish(),
        )
        .await
    {
        Ok(result) => result,
        Err(e) => {
            let mut std_out: std::io::Stdout = std::io::stdout();
            std_out.write_all(e.to_string().as_bytes()).unwrap();
            std_out.flush().unwrap();
            std::process::exit(1);
        }
    };

    let () = 'outer: loop {
        tokio::select! {
            value = rabbitmq_recv.recv() => {
                match value {
                    Some(message) => match message.content {
                        Some(buffer) => {
                            let transaction_dto: TransactionHandlerDTO =
                                serde_json::from_slice(&buffer).unwrap();

                            let user_balance: f64 = crate::database::helpers::transaction::get_user_balance(
                                &postgres,
                                &transaction_dto.from,
                            )
                            .await;

                            if transaction_dto.is_external {
                                let _ = postgres.execute_raw::<str, &str, Vec<&str>>(format!(
                                    r#"INSERT INTO transactions (from_user, to_user, user_id, amount, tx_status, is_external) VALUES (NULL, NULL, $1, {}, 'SUCCESS', TRUE)"#
                                , transaction_dto.amount).as_str(), vec![&transaction_dto.to]).await;

                                let sse_event_dto: super::ChannelDTO = super::ChannelDTO {
                                    user_id: transaction_dto.from.clone(),
                                    event_name: "Success".into(),
                                    event_data: Bytes::from(serde_json::to_string(&transaction_dto).unwrap())
                                        .to_vec(),
                                };

                                let _ = rabbitmq
                                    .basic_publish(
                                        BasicProperties::default()
                                            .with_content_type("application/json")
                                            .with_content_encoding("utf-8")
                                            .finish(),
                                        serde_json::to_vec(&sse_event_dto).unwrap(),
                                        BasicPublishArguments::default()
                                            .exchange("amq.fanout".into())
                                            .routing_key("events".into())
                                            .finish(),
                                    )
                                    .await;
                            } else {
                                if user_balance < transaction_dto.amount {
                                    let _ = postgres.execute_raw::<str, &str, Vec<&str>>(format!(
                                        r#"INSERT INTO transactions (from_user, to_user, user_id, amount, tx_status) VALUES ($1, $2, $3, {}, 'FAILED')"#
                                    , transaction_dto.amount).as_str(), vec![&transaction_dto.from, &transaction_dto.to, &transaction_dto.from]).await;

                                    let sse_event_dto: super::ChannelDTO = super::ChannelDTO {
                                        user_id: transaction_dto.from.clone(),
                                        event_name: "Failure".into(),
                                        event_data: Bytes::from(serde_json::to_string(&transaction_dto).unwrap())
                                            .to_vec(),
                                    };

                                    let _ = rabbitmq
                                        .basic_publish(
                                            BasicProperties::default()
                                                .with_content_type("application/json")
                                                .with_content_encoding("utf-8")
                                                .finish(),
                                            serde_json::to_vec(&sse_event_dto).unwrap(),
                                            BasicPublishArguments::default()
                                                .exchange("amq.fanout".into())
                                                .routing_key("events".into())
                                                .finish(),
                                        )
                                        .await;
                                } else {
                                    let _ = postgres.execute_raw::<str, &str, Vec<&str>>(format!(
                                        r#"INSERT INTO transactions (from_user, to_user, user_id, amount, tx_status) VALUES (NULL, $1, $2, {}, 'SUCCESS')"#
                                    , transaction_dto.amount).as_str(), vec![&transaction_dto.to, &transaction_dto.from]).await;

                                    let _ = postgres.execute_raw::<str, &str, Vec<&str>>(format!(
                                        r#"INSERT INTO transactions (from_user, to_user, user_id, amount, tx_status) VALUES ($1, NULL, $2, {}, 'RECEIVED')"#
                                    , transaction_dto.amount).as_str(), vec![&transaction_dto.from, &transaction_dto.to]).await;

                                    let sender_sse_event_dto: super::ChannelDTO = super::ChannelDTO {
                                        user_id: transaction_dto.from.clone(),
                                        event_name: "Success".into(),
                                        event_data: Bytes::from(serde_json::to_string(&transaction_dto).unwrap())
                                            .to_vec(),
                                    };

                                    let receiver_sse_event_dto: super::ChannelDTO = super::ChannelDTO {
                                        user_id: transaction_dto.to.clone(),
                                        event_name: "Receive".into(),
                                        event_data: Bytes::from(serde_json::to_string(&transaction_dto).unwrap())
                                            .to_vec(),
                                    };

                                    let _ = rabbitmq
                                        .basic_publish(
                                            BasicProperties::default()
                                                .with_content_type("application/json")
                                                .with_content_encoding("utf-8")
                                                .finish(),
                                            serde_json::to_vec(&sender_sse_event_dto).unwrap(),
                                            BasicPublishArguments::default()
                                                .exchange("amq.fanout".into())
                                                .routing_key("events".into())
                                                .finish(),
                                        )
                                        .await;

                                    let _ = rabbitmq
                                        .basic_publish(
                                            BasicProperties::default()
                                                .with_content_type("application/json")
                                                .with_content_encoding("utf-8")
                                                .finish(),
                                            serde_json::to_vec(&receiver_sse_event_dto).unwrap(),
                                            BasicPublishArguments::default()
                                                .exchange("amq.fanout".into())
                                                .routing_key("events".into())
                                                .finish(),
                                        )
                                        .await;
                                }
                            }
                        }

                        None => (),
                    },

                    None => (),
                }
            }

            _ = oneshot_receiver.recv() => {
                rabbitmq_recv.close();
                break 'outer;
            }
        }
    };
}
