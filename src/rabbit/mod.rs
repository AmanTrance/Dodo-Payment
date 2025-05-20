pub mod dto {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    pub(crate) struct ChannelDTO {
        pub(crate) user_id: String,
        pub(crate) event_name: String,
        pub(crate) event_data: Vec<u8>,
    }
}

use amqprs::{
    callbacks::{DefaultChannelCallback, DefaultConnectionCallback},
    channel::{Channel, QueueBindArguments, QueueDeclareArguments},
    connection::OpenConnectionArguments,
};

pub(crate) async fn open_rabbitmq_channel(
    host: &str,
    port: u16,
    username: &str,
    password: &str,
) -> Result<(Channel, amqprs::connection::Connection), amqprs::error::Error> {
    let options: OpenConnectionArguments =
        OpenConnectionArguments::new(host, port, username, password);

    let connection: amqprs::connection::Connection =
        amqprs::connection::Connection::open(&options).await?;

    connection
        .register_callback(DefaultConnectionCallback)
        .await?;

    let channel: Channel = connection.open_channel(None).await?;

    channel.register_callback(DefaultChannelCallback).await?;

    Ok((channel, connection))
}

pub(crate) async fn setup_channel_and_queues(
    channel: &Channel,
    queue_name: &str,
) -> Result<String, amqprs::error::Error> {
    let result: Option<(String, u32, u32)> = channel
        .queue_declare(
            QueueDeclareArguments::durable_client_named(queue_name)
                .no_wait(false)
                .finish(),
        )
        .await?;
    let _ = channel
        .queue_bind(QueueBindArguments::new(queue_name, "amq.direct", "dodo"))
        .await;

    Ok(result.unwrap().0)
}
