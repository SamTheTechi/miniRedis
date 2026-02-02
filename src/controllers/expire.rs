use crate::model::DB;
use anyhow::Result;
use std::time::{Duration, Instant};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

pub async fn expire_cmd(key: String, seconds: u64, _db: &DB, socket: &mut TcpStream) -> Result<()> {
    let mut db = _db.write().await;

    match db.get_mut(&key) {
        Some(entry) => {
            entry.expires_at = Some(Instant::now() + Duration::from_secs(seconds));
            socket.write_all(format!(":1\r\n").as_bytes()).await?;
        }
        None => {
            socket.write_all(format!(":0\r\n").as_bytes()).await?;
        }
    }

    Ok(())
}
