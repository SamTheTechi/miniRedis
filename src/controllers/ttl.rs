use crate::model::{DB, Heap, MinHeap};
use crate::util::is_expired;
use anyhow::Result;
use std::time::Instant;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

pub async fn ttl_cmd(
    key: String,
    _db: &DB,
    _heap: &mut Heap,
    socket: &mut TcpStream,
) -> Result<()> {
    let db = _db.read().await;

    match db.get(&key) {
        None => {
            socket.write_all(format!(":-2\r\n").as_bytes()).await?;
        }
        Some(entry) if is_expired(entry) => {
            let mut heap = _heap.lock().await;

            let val = MinHeap {
                key: key.clone(),
                expires_at: entry.expires_at.unwrap(),
            };

            heap.push(val);
            socket.write_all(format!(":-2\r\n").as_bytes()).await?;
        }
        Some(entry) => match entry.expires_at {
            None => {
                socket.write_all(format!(":-1\r\n").as_bytes()).await?;
            }
            Some(time) => {
                let ttl = time.saturating_duration_since(Instant::now()).as_secs();
                socket.write_all(format!(":{}\r\n", ttl).as_bytes()).await?;
            }
        },
    }

    Ok(())
}
