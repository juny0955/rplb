use std::{net::SocketAddr, sync::Arc};

use rplb::pool::{PoolError, ServerPool};
use tokio::{
    io,
    net::{TcpListener, TcpStream},
};

#[tokio::main]
async fn main() -> io::Result<()> {
    tracing_subscriber::fmt().with_target(false).init();

    let listen_addr = SocketAddr::from(([127, 0, 0, 1], 8080));

    let server_addrs = [
        SocketAddr::from(([127, 0, 0, 1], 9000)),
        SocketAddr::from(([127, 0, 0, 1], 9001)),
    ];
    let server_pool = Arc::new(ServerPool::new(server_addrs.to_vec()));

    let listener = TcpListener::bind(listen_addr).await?;

    tracing::info!(%listen_addr, ?server_addrs, "rplb started");

    loop {
        match listener.accept().await {
            Ok((client, client_addr)) => {
                let server_addr = match server_pool.select() {
                    Ok(addr) => addr,
                    Err(PoolError::EmptyPool) => {
                        tracing::warn!(%client_addr, "no server available");
                        continue;
                    }
                };

                tokio::spawn(async move {
                    if let Err(e) = handle_connection(client, server_addr).await {
                        tracing::warn!(
                            %client_addr,
                            %server_addr,
                            %e,
                            "proxy connection failed"
                        );
                    }
                });
            }
            Err(e) => {
                tracing::warn!(%e, "failed to accept client connection");
            }
        }
    }
}

async fn handle_connection(mut client: TcpStream, server_addr: SocketAddr) -> io::Result<()> {
    let mut server_stream = TcpStream::connect(server_addr).await?;
    io::copy_bidirectional(&mut client, &mut server_stream).await?;
    Ok(())
}
