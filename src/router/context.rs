use tokio::sync::mpsc::UnboundedSender;

use crate::EventHandlerDTO;

pub(crate) struct Context {
    pub(crate) postgres: tokio_postgres::Client,
    pub(crate) rabbitmq: amqprs::channel::Channel,
    pub(crate) register: UnboundedSender<EventHandlerDTO>,
}

impl Context {
    pub(crate) fn new(
        postgres: tokio_postgres::Client,
        rabbitmq: amqprs::channel::Channel,
        register: UnboundedSender<EventHandlerDTO>,
    ) -> Self {
        Self {
            postgres,
            rabbitmq,
            register,
        }
    }
}
