use crate::model::{DB, Heap, MinHeap};
use crate::util::is_expired;
use anyhow::Result;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

pub async fn exists_cmd(
    keys: Vec<String>,
    _db: &DB,
    _heap: &mut Heap,
    socket: &mut TcpStream,
) -> Result<()> {
    let mut removed_count = 0;
    let mut expired: Vec<(String, std::time::Instant)> = Vec::new();
    {
        let db = _db.read().await;

        for key in keys {
            if let Some(entry) = db.get(&key) {
                if is_expired(entry) {
                    if let Some(expires_at) = entry.expires_at {
                        expired.push((key, expires_at));
                    }
                } else {
                    removed_count += 1;
                }
            }
        }
    }

    if !expired.is_empty() {
        let mut heap = _heap.lock().await;
        for (key, expires_at) in expired {
            heap.push(MinHeap { key, expires_at });
        }
    }

    socket
        .write_all(format!(":{}\r\n", removed_count).as_bytes())
        .await?;
    Ok(())
}
