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
    channel::Channel,
    connection::OpenConnectionArguments,
};

pub(crate) async fn open_rabbitmq_channel(
    host: &str,
    port: u16,
    username: &str,
    password: &str,
) -> Result<Channel, amqprs::error::Error> {
    let options: OpenConnectionArguments =
        OpenConnectionArguments::new(host, port, username, password);

    let connection: amqprs::connection::Connection =
        amqprs::connection::Connection::open(&options).await?;

    connection
        .register_callback(DefaultConnectionCallback)
        .await?;

    let channel: Channel = connection.open_channel(None).await?;

    channel.register_callback(DefaultChannelCallback).await?;

    Ok(channel)
}
