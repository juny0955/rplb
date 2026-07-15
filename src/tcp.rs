use std::net::SocketAddr;

use tokio::{
    io,
    net::{TcpListener, TcpStream},
};

use crate::pool::{PoolError, ServerPool};

pub async fn serve(listener: TcpListener, server_pool: ServerPool) -> io::Result<()> {
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
