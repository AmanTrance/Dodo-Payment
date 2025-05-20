use std::{collections::HashMap, sync::Arc};

use amqprs::{BasicProperties, Deliver, channel::BasicAckArguments, consumer::AsyncConsumer};
use hyper::body::Bytes;
use tokio::sync::RwLock;

pub(crate) struct EventConsumer {
    map: Arc<RwLock<HashMap<String, tokio::sync::mpsc::Sender<Bytes>>>>,
}

impl<'life> EventConsumer {
    pub(crate) fn new(map: Arc<RwLock<HashMap<String, tokio::sync::mpsc::Sender<Bytes>>>>) -> Self {
        Self { map }
    }
}

#[async_trait::async_trait]
impl AsyncConsumer for EventConsumer {
    async fn consume(
        &mut self,
        channel: &amqprs::channel::Channel,
        deliver: Deliver,
        basic_properties: BasicProperties,
        content: Vec<u8>,
    ) {
        if basic_properties.content_type().is_some()
            && basic_properties.content_type().unwrap().to_lowercase() == "application/json"
        {
            let event_dto: crate::rabbit::dto::ChannelDTO =
                serde_json::from_slice(&content).unwrap();
            let _ = match self.map.read().await.get(&event_dto.user_id) {
                Some(sender) => {
                    sender
                        .send(Bytes::from(format!(
                            "event: {}\ndata: {}\n\n",
                            event_dto.event_name,
                            String::from_utf8(event_dto.event_data).unwrap()
                        )))
                        .await
                }

                None => Ok(()),
            };

            let _ = channel
                .basic_ack(BasicAckArguments::new(deliver.delivery_tag(), true))
                .await;
        }
    }
}
