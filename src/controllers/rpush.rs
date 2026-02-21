use crate::{
    lru::{estimate_entry_bytes, LruManager},
    model::{DB, Entry, Heap, Value},
};
use anyhow::Result;
use std::collections::VecDeque;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

pub async fn rpush_cmd(
    key: String,
    values: Vec<Vec<u8>>,
    _db: &DB,
    _heap: &mut Heap,
    lru: &LruManager,
    socket: &mut TcpStream,
) -> Result<()> {
    let inserted = values.len();
    let mut db = _db.write().await;
    let key_clone = key.clone();

    let (len, old_size, new_size, created_new) = match db.get_mut(&key) {
        Some(entry) => {
            let old_size = estimate_entry_bytes(&key, entry);
            let len = {
                let list = match entry.value.as_list_mut() {
                    Some(l) => l,
                    None => {
                        socket
                            .write_all(b"-WRONGTYPE key holds wrong kind of value\r\n")
                            .await?;
                        return Ok(());
                    }
                };
                for v in values {
                    list.push_back(v);
                }
                list.len()
            };
            let new_size = estimate_entry_bytes(&key, entry);
            (len, old_size, new_size, false)
        }
        None => {
            let mut list = VecDeque::new();
            for v in values {
                list.push_back(v);
            }

            let len = list.len();
            let new_entry = Entry {
                value: Value::List(list),
                expires_at: None,
            };
            let new_size = estimate_entry_bytes(&key, &new_entry);
            db.insert(key.clone(), new_entry);
            (len, 0usize, new_size, true)
        }
    };

    drop(db);
    let delta = new_size as isize - old_size as isize;
    let new_used = lru.adjust_used_bytes(delta);
    let maxmemory = lru.maxmemory();

    if maxmemory > 0 && new_used > maxmemory {
        let evicted = lru.evict_if_needed(_db, _heap).await?;
        if !evicted {
            let mut db = _db.write().await;
            if created_new {
                db.remove(&key_clone);
            } else if let Some(entry) = db.get_mut(&key_clone) {
                if let Some(list) = entry.value.as_list_mut() {
                    for _ in 0..inserted {
                        let _ = list.pop_back();
                    }
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

    socket.write_all(format!(":{}\r\n", len).as_bytes()).await?;

    Ok(())
}
