use tokio::{io, net::TcpListener};


#[tokio::main]
async fn main() -> io::Result<()>{
    println!("rplb started");
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("listening in 8080");


    match listener.accept().await {
        Ok((_socket, addr)) => println!("new client: {:?}", addr),
        Err(e) => println!("couldn't get clienct: {:?}", e),
    }

    Ok(())
}
