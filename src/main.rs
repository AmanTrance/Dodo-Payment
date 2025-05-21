mod database;
mod events;
mod handlers;
mod rabbit;
mod router;
mod utils;

use events::EventHandlerDTO;
use hyper::server::conn::http1;
use hyper_util::rt::TokioIo;
use std::{
    io::Write,
    net::{Ipv4Addr, SocketAddr},
};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::mpsc::unbounded_channel,
};

#[tokio::main]
async fn main() {
    let postgres_connection: tokio_postgres::Client =
        match database::connection::create_postgres_connection().await {
            Ok(client) => client,
            Err(e) => {
                let mut std_out: std::io::Stdout = std::io::stdout();
                std_out.write(e.to_string().as_bytes()).unwrap();
                std_out.flush().unwrap();
                std::process::exit(1);
            }
        };

    let transaction_postgres_connection: tokio_postgres::Client =
        match database::connection::create_postgres_connection().await {
            Ok(client) => client,
            Err(e) => {
                let mut std_out: std::io::Stdout = std::io::stdout();
                std_out.write(e.to_string().as_bytes()).unwrap();
                std_out.flush().unwrap();
                std::process::exit(1);
            }
        };

    let (transactions_channel_rabbitmq, events_channel_rabbitmq, connection) =
        match rabbit::open_rabbitmq_channel("localhost", 5672, "guest", "guest").await {
            Ok(result) => result,
            Err(e) => {
                let mut std_out: std::io::Stdout = std::io::stdout();
                std_out.write_all(e.to_string().as_bytes()).unwrap();
                std_out.flush().unwrap();
                std::process::exit(1);
            }
        };

    let (event_sender, event_receiver) = unbounded_channel::<EventHandlerDTO>();

    let transactions_queue_name = match crate::rabbit::setup_channel_and_queues(
        &transactions_channel_rabbitmq,
        "transactions",
        "amq.direct",
        "transactions",
    )
    .await
    {
        Ok(value) => value,
        Err(e) => {
            let mut std_out: std::io::Stdout = std::io::stdout();
            std_out.write_all(e.to_string().as_bytes()).unwrap();
            std_out.flush().unwrap();
            std::process::exit(1);
        }
    };

    let events_queue_name = match crate::rabbit::setup_channel_and_queues(
        &events_channel_rabbitmq,
        "events",
        "amq.fanout",
        "events",
    )
    .await
    {
        Ok(value) => value,
        Err(e) => {
            let mut std_out: std::io::Stdout = std::io::stdout();
            std_out.write_all(e.to_string().as_bytes()).unwrap();
            std_out.flush().unwrap();
            std::process::exit(1);
        }
    };

    tokio::spawn(crate::events::setup_event_handler(
        event_receiver,
        events_channel_rabbitmq,
        events_queue_name,
    ));

    tokio::spawn(crate::events::transaction::setup_transaction_handler(
        transaction_postgres_connection,
        transactions_channel_rabbitmq.clone(),
        transactions_queue_name,
    ));

    let router: std::sync::Arc<router::Router> = std::sync::Arc::<router::Router>::new(
        router::Router::new(
            postgres_connection,
            transactions_channel_rabbitmq,
            event_sender.clone(),
        )
        .await,
    );

    let address: SocketAddr =
        SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 11000);
    let listener: TcpListener = TcpListener::bind(address).await.unwrap();

    loop {
        tokio::select! {
            value = listener.accept() => {
                let mut client_fd: Option<TcpStream> = None;
                match value {
                    Ok(x) => {
                        client_fd = Some(x.0);
                        ()
                    }
                    Err(_) => (),
                };

                if client_fd.is_none() {
                    continue;
                } else {
                    let router_ptr: std::sync::Arc<router::Router> = std::sync::Arc::clone(&router);
                    tokio::task::spawn((async move || {
                        let client_stream: TokioIo<TcpStream> = TokioIo::new(client_fd.unwrap());
                        http1::Builder::new()
                            .keep_alive(true)
                            .ignore_invalid_headers(false)
                            .serve_connection(client_stream, router_ptr)
                            .await
                    })());
                }
            }

            _ = tokio::signal::ctrl_c() => {
                let _ = connection.close().await;
                event_sender.send(EventHandlerDTO::StopHandler).unwrap();
                tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                std::process::exit(0);
            }
        }
    }
}
