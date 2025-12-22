use crate::model::types;
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
        let first_byte = buf[0];

        match first_byte {
            b'+' => {}
            b'-' => {}
            b':' => {}
            b'$' => {}
            b'*' => {}
            _ => return Err(anyhow::anyhow!("Invalid RESP type")),
        }

        socket.write_all(b"+OK\r\n").await?;
    }
    Ok(())
}
