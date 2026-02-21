use crate::{
    lru::{estimate_entry_bytes, LruManager},
    model::{DB, Heap, MinHeap},
};
use crate::util::is_expired;
use anyhow::Result;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

pub async fn rpop_cmd(
    key: String,
    _db: &DB,
    _heap: &mut Heap,
    lru: &LruManager,
    socket: &mut TcpStream,
) -> Result<()> {
    let mut expired_at = None;
    let mut popped: Option<Vec<u8>> = None;
    let mut old_size = 0usize;
    let mut new_size = 0usize;
    let mut remove_key = false;
    let mut wrong_type = false;

    {
        let mut db = _db.write().await;
        if let Some(entry) = db.get_mut(&key) {
            if is_expired(entry) {
                expired_at = entry.expires_at;
            } else {
                old_size = estimate_entry_bytes(&key, entry);
                match entry.value.as_list_mut() {
                    Some(list) => {
                        popped = list.pop_back();
                        if list.is_empty() {
                            remove_key = true;
                        } else {
                            new_size = estimate_entry_bytes(&key, entry);
                        }
                    }
                    None => {
                        wrong_type = true;
                    }
                }

                if remove_key {
                    db.remove(&key);
                }
            }
        }
    }

    if wrong_type {
        socket
            .write_all(b"-WRONGTYPE key holds wrong kind of value\r\n")
            .await?;
        return Ok(());
    }

    if let Some(expires_at) = expired_at {
        let mut heap = _heap.lock().await;
        heap.push(MinHeap {
            key: key.clone(),
            expires_at,
        });
        socket.write_all(b"$-1\r\n").await?;
        return Ok(());
    }

    let delta = new_size as isize - old_size as isize;
    if delta != 0 {
        lru.adjust_used_bytes(delta);
    }
    if remove_key {
        lru.remove_key(&key).await;
    }

    match popped {
        Some(value) => {
            let mut resp = Vec::new();
            resp.extend_from_slice(b"$");
            resp.extend_from_slice(value.len().to_string().as_bytes());
            resp.extend_from_slice(b"\r\n");
            resp.extend_from_slice(&value);
            resp.extend_from_slice(b"\r\n");
            socket.write_all(&resp).await?;
        }
        None => {
            socket.write_all(b"$-1\r\n").await?;
        }
    }

    Ok(())
}
