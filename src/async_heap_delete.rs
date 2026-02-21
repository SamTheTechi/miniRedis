use crate::{
    lru::{estimate_entry_bytes, LruManager},
    model::{DB, Heap, MinHeap},
};
use std::time::Instant;
use tokio::time::{Duration, sleep};

pub fn async_clean_db_heap(mut _db: DB, mut _heap: Heap, lru: LruManager) {
    tokio::spawn(async move {
        loop {
            sleep(Duration::from_millis(100)).await;

            let now = Instant::now();

            let mut heap = _heap.lock().await;
            let mut db = _db.write().await;
            let mut removed_bytes = 0usize;
            let mut removed_keys: Vec<String> = Vec::new();

            while let Some(top) = heap.peek() {
                if top.expires_at > now {
                    break;
                }

                let MinHeap { expires_at, key } = heap.pop().unwrap();

                if let Some(entry) = db.get(&key) {
                    if entry.expires_at == Some(expires_at) {
                        if let Some((stored_key, removed_entry)) = db.remove_entry(&key) {
                            removed_bytes += estimate_entry_bytes(&stored_key, &removed_entry);
                            removed_keys.push(stored_key);
                        }
                    }
                }
            }

            drop(db);
            drop(heap);

            if removed_bytes > 0 {
                lru.adjust_used_bytes(-(removed_bytes as isize));
            }
            if !removed_keys.is_empty() {
                lru.remove_keys(&removed_keys).await;
            }
        }
    });
}
