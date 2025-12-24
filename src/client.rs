use crate::{model::types, parser::parse_resp};
use anyhow::{Ok, Result};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub async fn process_client(mut socket: TcpStream, mut _db: types::DB) -> Result<()> {
    let mut buf = vec![0u8; 4096];

    loop {
        let n = socket.read(&mut buf).await?;
        if n == 0 {
            println!("Client Disconnected");
            break;
        }

        let result = parse_resp(&mut buf).unwrap();
        println!("{:?}", result);
        socket.write_all(b"+OK\r\n").await?;
    }
    Ok(())
}
