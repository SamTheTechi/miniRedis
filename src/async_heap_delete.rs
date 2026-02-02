use crate::model::{DB, Heap, MinHeap};
use std::time::Instant;
use tokio::time::{Duration, sleep};

pub fn async_clean_db_heap(mut _db: DB, mut _heap: Heap) {
    tokio::spawn(async move {
        loop {
            sleep(Duration::from_millis(100)).await;

            let now = Instant::now();

            let mut heap = _heap.lock().await;
            let mut db = _db.write().await;

            while let Some(top) = heap.peek() {
                if top.expires_at > now {
                    break;
                }

                let MinHeap { expires_at, key } = heap.pop().unwrap();

                if let Some(entry) = db.get(&key) {
                    if entry.expires_at == Some(expires_at) {
                        db.remove(&key);
                    }
                }
            }
        }
    });
}
