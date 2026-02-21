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
    let mut expires_at = None;
    let mut resp: Option<Vec<u8>> = None;
    {
        let db = _db.read().await;
        match db.get(&key) {
            Some(entry) => {
                if is_expired(entry) {
                    expires_at = entry.expires_at;
                } else {
                    resp = Some(entry.value.to_resp_bytes());
                }
            }
            None => {}
        }
    }

    if let Some(expires_at) = expires_at {
        let mut heap = _heap.lock().await;
        heap.push(MinHeap {
            key: key.clone(),
            expires_at,
        });
        socket.write_all(b"$-1\r\n").await?;
        return Ok(());
    }

    match resp {
        Some(bytes) => socket.write_all(&bytes).await?,
        None => socket.write_all(b"$-1\r\n").await?,
    }

    Ok(())
}
