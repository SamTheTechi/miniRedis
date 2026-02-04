use crate::model::{DB, Entry, Value};
use anyhow::Result;
use std::time::{Duration, Instant};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

pub async fn psetex_cmd(
    key: String,
    value: Vec<u8>,
    seconds: u64,
    _db: &DB,
    socket: &mut TcpStream,
) -> Result<()> {
    let mut db = _db.write().await;
    db.insert(
        key,
        Entry {
            value: Value::String(value),
            expires_at: Some(Instant::now() + Duration::from_millis(seconds)),
        },
    );
    socket.write_all(b"+OK\r\n").await?;

    Ok(())
}
