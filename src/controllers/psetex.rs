use crate::{
    lru::{estimate_entry_bytes, LruManager},
    model::{DB, Entry, Heap, MinHeap, Value},
};
use anyhow::Result;
use std::time::{Duration, Instant};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

pub async fn psetex_cmd(
    key: String,
    value: Vec<u8>,
    seconds: u64,
    _db: &DB,
    _heap: &mut Heap,
    lru: &LruManager,
    socket: &mut TcpStream,
) -> Result<()> {
    let expires_at = Instant::now() + Duration::from_millis(seconds);
    let new_entry = Entry {
        value: Value::String(value),
        expires_at: Some(expires_at),
    };
    let new_size = estimate_entry_bytes(&key, &new_entry);

    let mut db = _db.write().await;
    let old = db.insert(key.clone(), new_entry);
    let old_size = old
        .as_ref()
        .map(|entry| estimate_entry_bytes(&key, entry))
        .unwrap_or(0);
    drop(db);

    let mut heap = _heap.lock().await;
    heap.push(MinHeap {
        key: key.clone(),
        expires_at,
    });
    drop(heap);

    let delta = new_size as isize - old_size as isize;
    let new_used = lru.adjust_used_bytes(delta);
    let maxmemory = lru.maxmemory();

    if maxmemory > 0 && new_used > maxmemory {
        let evicted = lru.evict_if_needed(_db, _heap).await?;
        if !evicted {
            let mut db = _db.write().await;
            match old {
                Some(old_entry) => {
                    db.insert(key.clone(), old_entry);
                }
                None => {
                    db.remove(&key);
                }
            }
            drop(db);
            lru.adjust_used_bytes(-delta);
            socket
                .write_all(b"-OOM command not allowed when used memory > 'maxmemory'.\r\n")
                .await?;
            return Ok(());
        }
    }

    socket.write_all(b"+OK\r\n").await?;

    Ok(())
}
