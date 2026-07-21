use std::{net::SocketAddr, num::NonZeroU32};

use rplb::{
    pool::{Backend, LoadBalancingPolicy, ServerPool},
    tcp::serve,
};
use tokio::{io, net::TcpListener};

#[tokio::main]
async fn main() -> io::Result<()> {
    tracing_subscriber::fmt().with_target(false).init();

    let listen_addr = SocketAddr::from(([127, 0, 0, 1], 8080));

    let server_addrs = [
        SocketAddr::from(([127, 0, 0, 1], 9000)),
        SocketAddr::from(([127, 0, 0, 1], 9001)),
    ];
    let backends = server_addrs.map(|addr| Backend::new(addr, NonZeroU32::MIN));
    let server_pool = ServerPool::new(Vec::from(backends), LoadBalancingPolicy::RR);

    let listener = TcpListener::bind(listen_addr).await?;

    tracing::info!(%listen_addr, ?server_addrs, "rplb started");

    serve(listener, server_pool).await
}
