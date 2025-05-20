mod consumer;

use std::{collections::HashMap, sync::Arc};

use amqprs::channel::BasicConsumeArguments;
use hyper::body::Bytes;
use tokio::sync::RwLock;

#[derive(Debug)]
pub(crate) enum EventHandlerDTO {
    RegisterSender {
        user_id: String,
        sender: tokio::sync::mpsc::Sender<Bytes>,
    },
    StopHandler,
}

pub(crate) async fn setup_event_handler(
    mut receiver: tokio::sync::mpsc::Receiver<EventHandlerDTO>,
    rabbit_channel: amqprs::channel::Channel,
    queue_name: &str,
) -> () {
    let map: Arc<RwLock<HashMap<String, tokio::sync::mpsc::Sender<Bytes>>>> =
        Arc::new(RwLock::new(HashMap::new()));
    let consumer_tag: String = uuid::Uuid::new_v4().to_string();
    let consumer_arguments: BasicConsumeArguments =
        BasicConsumeArguments::new(queue_name, &consumer_tag);

    'outer: loop {
        tokio::select! {
            event_dto = receiver.recv() => {
                match event_dto {
                    Some (dto) => {
                        match dto {
                            EventHandlerDTO::RegisterSender { user_id, sender } => {
                                let mut safe_map: tokio::sync::RwLockWriteGuard<'_, HashMap<String, tokio::sync::mpsc::Sender<Bytes>>> = map.write().await;
                                safe_map.insert(user_id, sender);
                            }

                            EventHandlerDTO::StopHandler => {
                                break 'outer;
                            }
                        }
                    }

                    None => ()
                }
            }

            _ = rabbit_channel.basic_consume(consumer::EventConsumer::new(Arc::clone(&map)), consumer_arguments.clone()) => ()
        }
    }

    ()
}
