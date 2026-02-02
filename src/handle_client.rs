use crate::{
    controllers::{del_cmd, exists_cmd, expire_cmd, get_cmd, set_cmd, ttl_cmd},
    model::{Command, DB, Heap, RESP},
    parser::{parse_command, parse_resp},
};
use anyhow::Result;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub async fn process_client(mut socket: TcpStream, mut _db: DB, mut _heap: Heap) -> Result<()> {
    let mut buf = vec![0u8; 4096];

    loop {
        let n = socket.read(&mut buf).await?;
        if n == 0 {
            println!("Client Disconnected ");
            break;
        }

        let (resp, consumed) = match parse_resp(&mut buf, &mut 0) {
            Ok(Some(r)) => r,
            Ok(None) => continue,
            Err(e) => {
                socket
                    .write_all(format!("-ERR {}\r\n", e).as_bytes())
                    .await?;
                continue;
            }
        };
        buf.drain(..consumed);

        let command_items = match resp {
            RESP::Arrays(items) => items,
            _ => {
                socket
                    .write_all(b"-ERR Protocal error: expected array\r\n")
                    .await?;
                continue;
            }
        };

        let command = match parse_command(command_items) {
            Ok(c) => c,
            Err(e) => {
                socket
                    .write_all(format!("-ERR {}\r\n", e).as_bytes())
                    .await?;
                continue;
            }
        };

        match command {
            Command::PING => socket.write_all(b"+PONG\r\n").await?,
            Command::SET { key, value } => set_cmd(key, value, &_db, &mut socket).await?,
            Command::GET { key } => get_cmd(key, &_db, &mut _heap, &mut socket).await?,
            Command::DEL { keys } => del_cmd(keys, &_db, &mut socket).await?,
            Command::EXISTS { keys } => exists_cmd(keys, &_db, &mut socket).await?,
            Command::EXPIRE { key, sec } => expire_cmd(key, sec, &_db, &mut socket).await?,
            Command::TTL { key } => ttl_cmd(key, &_db, &mut _heap, &mut socket).await?,
        }
    }
    Ok(())
}
