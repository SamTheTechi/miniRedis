use crate::model::{DB, Heap, MinHeap, Value};
use crate::util::is_expired;
use anyhow::Result;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

pub async fn type_cmd(
    key: String,
    _db: &DB,
    _heap: &mut Heap,
    socket: &mut TcpStream,
) -> Result<()> {
    let mut expired_at = None;
    let mut value_type: Option<&'static str> = None;

    {
        let db = _db.read().await;
        if let Some(entry) = db.get(&key) {
            if is_expired(entry) {
                expired_at = entry.expires_at;
            } else {
                value_type = Some(match entry.value {
                    Value::String(_) => "string",
                    Value::List(_) => "list",
                });
            }
        }
    }

    if let Some(expires_at) = expired_at {
        let mut heap = _heap.lock().await;
        heap.push(MinHeap { key, expires_at });
        socket.write_all(b"+none\r\n").await?;
        return Ok(());
    }

    match value_type {
        Some(t) => socket.write_all(format!("+{}\r\n", t).as_bytes()).await?,
        None => socket.write_all(b"+none\r\n").await?,
    }

    Ok(())
}
