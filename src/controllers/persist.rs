use crate::model::{DB, Heap, MinHeap};
use crate::util::is_expired;
use anyhow::Result;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

pub async fn persist_cmd(
    key: String,
    _db: &DB,
    _heap: &mut Heap,
    socket: &mut TcpStream,
) -> Result<()> {
    let mut expired_at = None;
    let mut removed = false;

    {
        let mut db = _db.write().await;
        if let Some(entry) = db.get_mut(&key) {
            if is_expired(entry) {
                expired_at = entry.expires_at;
            } else if entry.expires_at.is_some() {
                entry.expires_at = None;
                removed = true;
            }
        }
    }

    if let Some(expires_at) = expired_at {
        let mut heap = _heap.lock().await;
        heap.push(MinHeap { key, expires_at });
        socket.write_all(b":0\r\n").await?;
        return Ok(());
    }

    if removed {
        socket.write_all(b":1\r\n").await?;
    } else {
        socket.write_all(b":0\r\n").await?;
    }

    Ok(())
}
