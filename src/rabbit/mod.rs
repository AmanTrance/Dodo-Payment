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
) -> Result<(Channel, Channel, amqprs::connection::Connection), amqprs::error::Error> {
    let options: OpenConnectionArguments =
        OpenConnectionArguments::new(host, port, username, password);

    let connection: amqprs::connection::Connection =
        amqprs::connection::Connection::open(&options).await?;

    connection
        .register_callback(DefaultConnectionCallback)
        .await?;

    let transactions_channel: Channel = connection.open_channel(None).await?;
    let events_channel: Channel = connection.open_channel(None).await?;

    transactions_channel
        .register_callback(DefaultChannelCallback)
        .await?;
    events_channel
        .register_callback(DefaultChannelCallback)
        .await?;

    Ok((transactions_channel, events_channel, connection))
}

pub(crate) async fn setup_channel_and_queues(
    channel: &Channel,
    queue_name: &str,
    exchange: &str,
    routing_key: &str,
) -> Result<String, amqprs::error::Error> {
    let result: Option<(String, u32, u32)> = channel
        .queue_declare(
            QueueDeclareArguments::durable_client_named(queue_name)
                .no_wait(false)
                .finish(),
        )
        .await?;
    let _ = channel
        .queue_bind(QueueBindArguments::new(queue_name, exchange, routing_key))
        .await;

    Ok(result.unwrap().0)
}
