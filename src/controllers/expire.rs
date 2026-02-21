use crate::model::{DB, Heap, MinHeap};
use anyhow::Result;
use std::time::{Duration, Instant};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

pub async fn expire_cmd(
    key: String,
    seconds: u64,
    _db: &DB,
    _heap: &mut Heap,
    socket: &mut TcpStream,
) -> Result<()> {
    let mut db = _db.write().await;

    match db.get_mut(&key) {
        Some(entry) => {
            let expires_at = Instant::now() + Duration::from_secs(seconds);
            entry.expires_at = Some(expires_at);
            socket.write_all(format!(":1\r\n").as_bytes()).await?;
            drop(db);

            let mut heap = _heap.lock().await;
            heap.push(MinHeap { key, expires_at });
            return Ok(());
        }
        None => {
            socket.write_all(format!(":0\r\n").as_bytes()).await?;
        }
    }

    Ok(())
}
