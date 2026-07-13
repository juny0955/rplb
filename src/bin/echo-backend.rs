use std::net::SocketAddr;

use tokio::{io::{self, AsyncReadExt, AsyncWriteExt}, net::{TcpListener, TcpStream}};


#[tokio::main]
async fn main() -> io::Result<()> {
     let address = SocketAddr::from(([127, 0, 0, 1], 9000));
     let listener = TcpListener::bind(address).await?;

     println!("echo backend listening on {address}");

     loop {
         let (stream, peer_addr) = listener.accept().await?;

         tokio::spawn(async move {
             if let Err(error) = echo(stream, peer_addr).await {
                 eprintln!("echo connection failed: {error}");
             }
         });
     }
 }

 async fn echo(mut stream: TcpStream, peer_addr: SocketAddr) -> io::Result<()> {
     let mut buffer = [0_u8; 4096];

     loop {
         let size = stream.read(&mut buffer).await?;

         if size == 0 {
             println!("client disconnected: {peer_addr}");
             return Ok(());
         }

         println!("received from {peer_addr}: {}", String::from_utf8_lossy(&buffer[..size]));

         stream.write_all(&buffer[..size]).await?;
     }
 }
