use crate::{
    controllers,
    lru::LruManager,
    model::{Command, DB, Heap, RESP},
    parser::{parse_command, parse_resp},
};
use anyhow::Result;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub async fn process_client(
    mut socket: TcpStream,
    mut _db: DB,
    mut _heap: Heap,
    lru: LruManager,
) -> Result<()> {
    let mut read_buf: Vec<u8> = Vec::new();
    let mut tmp = [0u8; 4096];
    let mut access_buffer: Vec<String> = Vec::new();

    loop {
        let n = socket.read(&mut tmp).await?;
        if n == 0 {
            println!("Client Disconnected ");
            break;
        }

        read_buf.extend_from_slice(&tmp[..n]);

        loop {
            if read_buf.is_empty() {
                break;
            }

            let first = read_buf[0];
            if !matches!(first, b'+' | b'-' | b':' | b'$' | b'*') {
                socket
                    .write_all(b"-ERR Protocol error: expected array\r\n")
                    .await?;
                read_buf.clear();
                break;
            }

            let mut offset = 0usize;
            let (resp, consumed) = match parse_resp(&read_buf, &mut offset) {
                Ok(Some(r)) => r,
                Ok(None) => break,
                Err(e) => {
                    socket
                        .write_all(format!("-ERR {}\r\n", e).as_bytes())
                        .await?;
                    read_buf.clear();
                    break;
                }
            };
            read_buf.drain(..consumed);

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
                    println!("Command parse error: {e}");
                    socket
                        .write_all(format!("-ERR {}\r\n", e).as_bytes())
                        .await?;
                    continue;
                }
            };

            println!("Request: {:?}", command);

            match command {
                Command::PING => socket.write_all(b"+PONG\r\n").await?,
                Command::QUIT => {
                    socket.write_all(b"+OK\r\n").await?;
                    break;
                }
                Command::SET { key, value } => {
                    lru.record_access(&mut access_buffer, &key);
                    controllers::set_cmd(key, value, &_db, &mut _heap, &lru, &mut socket).await?
                }
                Command::SETEX {
                    key,
                    value,
                    seconds,
                } => {
                    lru.record_access(&mut access_buffer, &key);
                    controllers::setex_cmd(key, value, seconds, &_db, &mut _heap, &lru, &mut socket)
                        .await?
                }
                Command::PSETEX {
                    key,
                    value,
                    seconds,
                } => {
                    lru.record_access(&mut access_buffer, &key);
                    controllers::psetex_cmd(
                        key,
                        value,
                        seconds,
                        &_db,
                        &mut _heap,
                        &lru,
                        &mut socket,
                    )
                    .await?
                }
                Command::GET { key } => {
                    lru.record_access(&mut access_buffer, &key);
                    controllers::get_cmd(key, &_db, &mut _heap, &mut socket).await?
                }
                Command::DEL { keys } => {
                    for key in &keys {
                        lru.record_access(&mut access_buffer, key);
                    }
                    controllers::del_cmd(keys, &_db, &lru, &mut socket).await?
                }
                Command::EXISTS { keys } => {
                    for key in &keys {
                        lru.record_access(&mut access_buffer, key);
                    }
                    controllers::exists_cmd(keys, &_db, &mut _heap, &mut socket).await?
                }
                Command::EXPIRE { key, seconds } => {
                    lru.record_access(&mut access_buffer, &key);
                    controllers::expire_cmd(key, seconds, &_db, &mut _heap, &mut socket).await?
                }
                Command::PERSIST { key } => {
                    lru.record_access(&mut access_buffer, &key);
                    controllers::persist_cmd(key, &_db, &mut _heap, &mut socket).await?
                }
                Command::TTL { key } => {
                    lru.record_access(&mut access_buffer, &key);
                    controllers::ttl_cmd(key, &_db, &mut _heap, &mut socket).await?
                }
                Command::PTTL { key } => {
                    lru.record_access(&mut access_buffer, &key);
                    controllers::pttl_cmd(key, &_db, &mut _heap, &mut socket).await?
                }
                Command::TYPE { key } => {
                    lru.record_access(&mut access_buffer, &key);
                    controllers::type_cmd(key, &_db, &mut _heap, &mut socket).await?
                }
                Command::INFO { section } => {
                    controllers::info_cmd(section, &_db, &lru, &mut socket).await?
                }
                Command::ConfigGet { pattern } => {
                    controllers::config_get_cmd(pattern, &lru, &mut socket).await?
                }
                Command::ConfigSet { key, value } => {
                    controllers::config_set_cmd(key, value, &lru, &mut socket).await?
                }
                Command::HELLO { version } => controllers::hello_cmd(version, &mut socket).await?,
                Command::COMMAND => controllers::command_cmd(&mut socket).await?,
                Command::ClientSetinfo => socket.write_all(b"+OK\r\n").await?,
                Command::LPUSH { key, values } => {
                    lru.record_access(&mut access_buffer, &key);
                    controllers::lpush_cmd(key, values, &_db, &mut _heap, &lru, &mut socket).await?
                }
                Command::RPUSH { key, values } => {
                    lru.record_access(&mut access_buffer, &key);
                    controllers::rpush_cmd(key, values, &_db, &mut _heap, &lru, &mut socket).await?
                }
                Command::LPOP { key } => {
                    lru.record_access(&mut access_buffer, &key);
                    controllers::lpop_cmd(key, &_db, &mut _heap, &lru, &mut socket).await?
                }
                Command::RPOP { key } => {
                    lru.record_access(&mut access_buffer, &key);
                    controllers::rpop_cmd(key, &_db, &mut _heap, &lru, &mut socket).await?
                }
            }
            lru.flush_accesses(&mut access_buffer);
        }
    }

    lru.flush_accesses(&mut access_buffer);
    Ok(())
}
