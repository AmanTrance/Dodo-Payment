mod database;
mod events;
mod handlers;
mod rabbit;
mod router;
mod utils;

use hyper::server::conn::http1;
use hyper_util::rt::TokioIo;
use std::{
    io::Write,
    net::{Ipv4Addr, SocketAddr},
};
use tokio::net::{TcpListener, TcpStream};

#[tokio::main(flavor = "current_thread")]
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

    let rabbitmq_sender_channel: amqprs::channel::Channel =
        match rabbit::open_rabbitmq_channel("localhost", 5672, "guest", "guest").await {
            Ok(channel) => channel,
            Err(e) => {
                let mut std_out: std::io::Stdout = std::io::stdout();
                std_out.write_all(e.to_string().as_bytes()).unwrap();
                std_out.flush().unwrap();
                std::process::exit(1);
            }
        };

    let router: std::sync::Arc<router::Router> = std::sync::Arc::<router::Router>::new(
        router::Router::new(postgres_connection, rabbitmq_sender_channel).await,
    );

    let address: SocketAddr =
        SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 11000);
    let listener: TcpListener = TcpListener::bind(address).await.unwrap();

    loop {
        let mut client_fd: Option<TcpStream> = None;
        let () = match listener.accept().await {
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
}
