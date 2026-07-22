use std::{io, net::SocketAddr, num::NonZeroU32};

use rplb::{
    pool::{LoadBalancingPolicy, Server, ServerPool},
    tcp::serve,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    task::JoinHandle,
};

fn weighted_server(addr: SocketAddr, weight: NonZeroU32) -> Server {
    Server::new(addr, weight)
}

struct Proxy {
    address: SocketAddr,
    task: JoinHandle<io::Result<()>>,
}

impl Drop for Proxy {
    fn drop(&mut self) {
        self.task.abort();
    }
}

async fn start_proxy(servers: Vec<Server>, policy: LoadBalancingPolicy) -> io::Result<Proxy> {
    let listener = TcpListener::bind(("127.0.0.1", 0)).await?;
    let address = listener.local_addr()?;
    let task = tokio::spawn(serve(listener, ServerPool::new(servers, policy)));

    Ok(Proxy { address, task })
}

async fn start_reply_server(
    reply: &'static [u8],
    connection_count: usize,
) -> io::Result<(SocketAddr, JoinHandle<io::Result<()>>)> {
    let listener = TcpListener::bind(("127.0.0.1", 0)).await?;
    let address = listener.local_addr()?;
    let task = tokio::spawn(async move {
        for _ in 0..connection_count {
            let (mut stream, _) = listener.accept().await?;
            stream.write_all(reply).await?;
            stream.shutdown().await?;
        }

        Ok(())
    });

    Ok((address, task))
}

async fn receive_reply(proxy_addr: SocketAddr) -> io::Result<Vec<u8>> {
    let mut client = TcpStream::connect(proxy_addr).await?;
    let mut reply = Vec::new();
    client.read_to_end(&mut reply).await?;

    Ok(reply)
}

async fn join(task: JoinHandle<io::Result<()>>) -> io::Result<()> {
    task.await.map_err(io::Error::other)?
}

#[tokio::test]
async fn 연속_연결을_서버에_라운드로빈_순서로_전달한다() -> io::Result<()> {
    // Given
    let (first_addr, first_server) = start_reply_server(b"A", 2).await?;
    let (second_addr, second_server) = start_reply_server(b"B", 1).await?;
    let proxy = start_proxy(
        vec![
            weighted_server(first_addr, NonZeroU32::MIN),
            weighted_server(second_addr, NonZeroU32::MIN),
        ],
        LoadBalancingPolicy::RR,
    )
    .await?;

    // When
    let replies = [
        receive_reply(proxy.address).await?,
        receive_reply(proxy.address).await?,
        receive_reply(proxy.address).await?,
    ];

    // Then
    assert_eq!(replies, [b"A".to_vec(), b"B".to_vec(), b"A".to_vec()]);
    join(first_server).await?;
    join(second_server).await?;

    Ok(())
}

#[tokio::test]
async fn 가중치가_3대1이면_연속_연결을_swrr_순서로_전달한다() -> io::Result<()> {
    // Given
    let (first_addr, first_server) = start_reply_server(b"A", 3).await?;
    let (second_addr, second_server) = start_reply_server(b"B", 1).await?;
    let proxy = start_proxy(
        vec![
            weighted_server(
                first_addr,
                NonZeroU32::new(3).expect("weight should be non-zero"),
            ),
            weighted_server(second_addr, NonZeroU32::MIN),
        ],
        LoadBalancingPolicy::SWRR,
    )
    .await?;

    // When
    let replies = [
        receive_reply(proxy.address).await?,
        receive_reply(proxy.address).await?,
        receive_reply(proxy.address).await?,
        receive_reply(proxy.address).await?,
    ];

    // Then
    assert_eq!(
        replies,
        [b"A".to_vec(), b"A".to_vec(), b"B".to_vec(), b"A".to_vec()]
    );
    join(first_server).await?;
    join(second_server).await?;

    Ok(())
}

#[tokio::test]
async fn 클라이언트_쓰기_종료_후에도_서버_응답을_전달한다() -> io::Result<()> {
    // Given
    let listener = TcpListener::bind(("127.0.0.1", 0)).await?;
    let server_addr = listener.local_addr()?;
    let server = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await?;
        let mut request = Vec::new();
        stream.read_to_end(&mut request).await?;

        if request != b"request" {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "unexpected proxy request",
            ));
        }

        stream.write_all(b"response").await?;
        stream.shutdown().await?;
        Ok(())
    });
    let proxy = start_proxy(
        vec![weighted_server(server_addr, NonZeroU32::MIN)],
        LoadBalancingPolicy::RR,
    )
    .await?;

    // When
    let mut client = TcpStream::connect(proxy.address).await?;
    client.write_all(b"request").await?;
    client.shutdown().await?;

    let mut reply = Vec::new();
    client.read_to_end(&mut reply).await?;

    // Then
    assert_eq!(reply, b"response");
    join(server).await?;

    Ok(())
}

#[tokio::test]
async fn 실패한_서버_연결_후에도_다음_연결을_처리한다() -> io::Result<()> {
    // Given
    let (healthy_addr, healthy_server) = start_reply_server(b"healthy", 1).await?;
    let unavailable_addr = SocketAddr::from(([127, 0, 0, 1], 0));
    let proxy = start_proxy(
        vec![
            weighted_server(unavailable_addr, NonZeroU32::MIN),
            weighted_server(healthy_addr, NonZeroU32::MIN),
        ],
        LoadBalancingPolicy::RR,
    )
    .await?;

    // When
    let mut failed_client = TcpStream::connect(proxy.address).await?;
    let mut first_buffer = [0_u8; 1];
    let first_read_size = failed_client.read(&mut first_buffer).await?;

    let next_reply = receive_reply(proxy.address).await?;

    // Then
    assert_eq!(first_read_size, 0);
    assert_eq!(next_reply, b"healthy");
    join(healthy_server).await?;

    Ok(())
}

#[tokio::test]
async fn 빈_풀이면_연속_클라이언트_연결을_종료하고_accept_loop을_유지한다() -> io::Result<()> {
    // Given
    let proxy = start_proxy(Vec::new(), LoadBalancingPolicy::RR).await?;

    // When
    let replies = [
        receive_reply(proxy.address).await?,
        receive_reply(proxy.address).await?,
    ];

    // Then
    assert_eq!(replies, [Vec::new(), Vec::new()]);

    Ok(())
}
