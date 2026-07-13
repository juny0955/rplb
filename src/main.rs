use std::net::SocketAddr;

use tokio::{
    io,
    net::{TcpListener, TcpStream},
};

#[tokio::main]
async fn main() -> io::Result<()> {
    tracing_subscriber::fmt().with_target(false).init();

    let listen_addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    let backend_addr = SocketAddr::from(([127, 0, 0, 1], 9000));
    let listener = TcpListener::bind(listen_addr).await?;

    tracing::info!(%listen_addr, %backend_addr, "rplb started");

    loop {
        match listener.accept().await {
            Ok((client, client_addr)) => {
                tokio::spawn(async move {
                    if let Err(e) = handle_connection(client, backend_addr).await {
                        tracing::warn!(%client_addr, %backend_addr, %e, "proxy connection failed");
                    }
                });
            }
            Err(e) => {
                tracing::warn!(%e, "failed to accept client connection");
            }
        }
    }
}

async fn handle_connection(mut client: TcpStream, backend_addr: SocketAddr) -> io::Result<()> {
    let mut backend = TcpStream::connect(backend_addr).await?;
    io::copy_bidirectional(&mut client, &mut backend).await?;
    Ok(())
}
