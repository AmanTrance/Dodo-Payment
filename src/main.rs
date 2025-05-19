mod database;
mod handlers;
mod router;
mod utils;

use hyper::server::conn::http1;
use hyper_util::rt::TokioIo;
use std::net::{Ipv4Addr, SocketAddr};
use tokio::net::{TcpListener, TcpStream};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let address: SocketAddr =
        SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 11000);
    let listener: TcpListener = TcpListener::bind(address).await.unwrap();
    let router: std::sync::Arc<router::Router> =
        std::sync::Arc::<router::Router>::new(router::Router::new().await);

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
