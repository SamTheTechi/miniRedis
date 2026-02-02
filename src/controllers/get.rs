use crate::model::{DB, Heap, MinHeap};
use crate::util::is_expired;
use anyhow::Result;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

pub async fn get_cmd(
    key: String,
    _db: &DB,
    _heap: &mut Heap,
    socket: &mut TcpStream,
) -> Result<()> {
    let db = _db.read().await;

    match db.get(&key) {
        Some(entry) => {
            if is_expired(&entry) {
                let mut heap = _heap.lock().await;

                let val = MinHeap {
                    key: key.clone(),
                    expires_at: entry.expires_at.unwrap(),
                };

                heap.push(val);

                socket.write_all(b"$-1\r\n").await?;
            }

            let resp = entry.value.to_resp_bytes();
            socket.write_all(&resp).await?;
        }
        None => {
            socket.write_all(b"$-1\r\n").await?;
        }
    }

    Ok(())
}
