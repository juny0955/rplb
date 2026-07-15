use std::net::SocketAddr;

use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

const DEFAULT_PORT: u16 = 9000;

fn listen_address(port: Option<&str>) -> io::Result<Option<SocketAddr>> {
    let port = match port {
        None => DEFAULT_PORT,
        Some("-h" | "--help") => return Ok(None),
        Some(port) => port
            .parse()
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))?,
    };

    Ok(Some(SocketAddr::from(([127, 0, 0, 1], port))))
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let argument = std::env::args().nth(1);
    let Some(address) = listen_address(argument.as_deref())? else {
        println!("Usage: echo-backend [PORT]");
        return Ok(());
    };
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

        println!(
            "received from {peer_addr}: {}",
            String::from_utf8_lossy(&buffer[..size])
        );

        stream.write_all(&buffer[..size]).await?;
    }
}

#[cfg(test)]
mod tests {
    use std::{io::ErrorKind, net::SocketAddr};

    use super::listen_address;

    #[test]
    fn 포트를_입력하면_해당_주소를_반환한다() {
        // Given
        let port = Some("9001");

        // When
        let address = listen_address(port).expect("valid port should parse");

        // Then
        assert_eq!(address, Some(SocketAddr::from(([127, 0, 0, 1], 9001))));
    }

    #[test]
    fn 포트를_입력하지_않으면_기본_주소를_반환한다() {
        // Given
        let port = None;

        // When
        let address = listen_address(port).expect("default address should resolve");

        // Then
        assert_eq!(address, Some(SocketAddr::from(([127, 0, 0, 1], 9000))));
    }

    #[test]
    fn 포트가_범위를_벗어나면_입력_오류를_반환한다() {
        // Given
        let port = Some("65536");

        // When
        let result = listen_address(port);

        // Then
        assert!(matches!(result, Err(error) if error.kind() == ErrorKind::InvalidInput));
    }
}
