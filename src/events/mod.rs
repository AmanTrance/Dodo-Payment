pub mod transaction;

use std::collections::HashMap;

use amqprs::channel::BasicConsumeArguments;
use hyper::body::Bytes;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct ChannelDTO {
    pub(crate) user_id: String,
    pub(crate) event_name: String,
    pub(crate) event_data: Vec<u8>,
}

#[derive(Debug)]
pub(crate) enum EventHandlerDTO {
    RegisterSender {
        user_id: String,
        sender: tokio::sync::mpsc::Sender<Bytes>,
    },
    StopHandler,
}

pub(crate) async fn setup_event_handler(
    mut receiver: tokio::sync::mpsc::UnboundedReceiver<EventHandlerDTO>,
    rabbit_channel: amqprs::channel::Channel,
    queue_name: String,
) -> () {
    let mut map: HashMap<String, tokio::sync::mpsc::Sender<Bytes>> = HashMap::new();
    let consumer_tag: String = uuid::Uuid::new_v4().to_string();
    let consumer_arguments: BasicConsumeArguments =
        BasicConsumeArguments::new(&queue_name, &consumer_tag);

    let (_, mut rabbit_receiver) = match rabbit_channel.basic_consume_rx(consumer_arguments).await {
        Ok(result) => result,
        Err(_) => std::process::exit(0),
    };

    'outer: loop {
        tokio::select! {
            event_dto = receiver.recv() => {
                match event_dto {
                    Some (dto) => {
                        match dto {
                            EventHandlerDTO::RegisterSender { user_id, sender } => {
                                map.insert(user_id, sender);
                            }

                            EventHandlerDTO::StopHandler => {
                                break 'outer;
                            }
                        }
                    }

                    None => ()
                }
            }

            content = rabbit_receiver.recv() => {
                match content {
                    Some(message) => {
                        match message.content {
                            Some(value) => {
                                let event_dto: ChannelDTO = serde_json::from_slice(&value).unwrap();
                                match map.get(&event_dto.user_id) {
                                    Some(sender) => {
                                        let _ = sender.send(Bytes::from(format!(
                                            "event: {}\ndata: {}\n\n"
                                        , event_dto.event_name, String::from_utf8(event_dto.event_data).unwrap()))).await;
                                    }

                                    None => ()
                                }
                            }

                            None => ()
                        }
                    }

                    None => ()
                }
            }

            _ = tokio::time::sleep(tokio::time::Duration::from_secs(10)) => {
                let mut unused_ids: Vec<String> = vec![];
                for (key, value) in map.iter() {
                    match value.send(Bytes::from(
                        "event: ping\ndata: {\"status\":\"alive\"}\n\n"
                    )).await {
                        Ok(_) => (),

                        Err(_) => {
                            unused_ids.push(key.clone());
                            ()
                        }
                    }
                }

                for id in unused_ids.iter() {
                    let _ = map.remove(id);
                }
            }
        }
    }

    ()
}
