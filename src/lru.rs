use crate::model::{DB, Entry, Value};
use anyhow::Result;
use std::{
    collections::{HashMap, VecDeque},
    mem::{self, size_of},
    sync::{
        Arc,
        atomic::{AtomicU8, AtomicU64, AtomicUsize, Ordering},
    },
};
use tokio::sync::{Mutex, mpsc};

const ACCESS_BATCH_SIZE: usize = 32;
const ACCESS_CHANNEL_CAPACITY: usize = 1024;
const SAMPLE_SIZE: usize = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EvictionPolicy {
    NoEviction,
    AllKeysLru,
    VolatileTtl,
}

#[derive(Clone)]
pub struct LruManager {
    access_tx: mpsc::Sender<Vec<String>>,
    last_access: Arc<Mutex<HashMap<String, u64>>>,
    maxmemory: Arc<AtomicUsize>,
    policy: Arc<AtomicU8>,
    used_bytes: Arc<AtomicUsize>,
}

impl LruManager {
    pub fn new(maxmemory: usize, policy: EvictionPolicy) -> Self {
        let (access_tx, mut access_rx) = mpsc::channel::<Vec<String>>(ACCESS_CHANNEL_CAPACITY);
        let last_access = Arc::new(Mutex::new(HashMap::<String, u64>::new()));
        let counter = Arc::new(AtomicU64::new(0));
        let maxmemory = Arc::new(AtomicUsize::new(maxmemory));
        let policy = Arc::new(AtomicU8::new(policy_to_u8(policy)));
        let used_bytes = Arc::new(AtomicUsize::new(0));

        let last_access_task = last_access.clone();
        let counter_task = counter.clone();
        let maxmemory_task = maxmemory.clone();
        let policy_task = policy.clone();

        tokio::spawn(async move {
            while let Some(batch) = access_rx.recv().await {
                if batch.is_empty() {
                    continue;
                }

                let current_policy = u8_to_policy(policy_task.load(Ordering::Relaxed));
                let current_maxmemory = maxmemory_task.load(Ordering::Relaxed);
                if current_policy != EvictionPolicy::AllKeysLru || current_maxmemory == 0 {
                    continue;
                }

                let mut map = last_access_task.lock().await;
                for key in batch {
                    let tick = counter_task.fetch_add(1, Ordering::Relaxed) + 1;
                    map.insert(key, tick);
                }
            }
        });

        Self {
            access_tx,
            last_access,
            maxmemory,
            policy,
            used_bytes,
        }
    }

    pub fn record_access(&self, buffer: &mut Vec<String>, key: &str) {
        if self.policy() != EvictionPolicy::AllKeysLru || self.maxmemory() == 0 {
            return;
        }
        buffer.push(key.to_string());
        if buffer.len() >= ACCESS_BATCH_SIZE {
            self.flush_accesses(buffer);
        }
    }

    pub fn flush_accesses(&self, buffer: &mut Vec<String>) {
        if self.policy() != EvictionPolicy::AllKeysLru || self.maxmemory() == 0 {
            buffer.clear();
            return;
        }
        if buffer.is_empty() {
            return;
        }
        let batch = mem::take(buffer);
        let _ = self.access_tx.try_send(batch);
    }

    pub async fn remove_key(&self, key: &str) {
        if self.policy() != EvictionPolicy::AllKeysLru {
            return;
        }
        let mut map = self.last_access.lock().await;
        map.remove(key);
    }

    pub async fn remove_keys(&self, keys: &[String]) {
        if self.policy() != EvictionPolicy::AllKeysLru {
            return;
        }
        if keys.is_empty() {
            return;
        }
        let mut map = self.last_access.lock().await;
        for key in keys {
            map.remove(key);
        }
    }

    pub async fn evict_if_needed(&self, db: &DB, heap: &crate::model::Heap) -> Result<bool> {
        let maxmemory = self.maxmemory();
        if maxmemory == 0 {
            return Ok(true);
        }
        let mut used = self.used_bytes.load(Ordering::Relaxed);
        if used <= maxmemory {
            return Ok(true);
        }

        match self.policy() {
            EvictionPolicy::NoEviction => return Ok(false),
            EvictionPolicy::AllKeysLru => {
                while used > maxmemory {
                    let sample_keys = {
                        let db_read = db.read().await;
                        db_read
                            .keys()
                            .take(SAMPLE_SIZE)
                            .cloned()
                            .collect::<Vec<_>>()
                    };

                    if sample_keys.is_empty() {
                        break;
                    }

                    let candidate = {
                        let map = self.last_access.lock().await;
                        let mut best: Option<(String, u64)> = None;

                        for key in &sample_keys {
                            let tick = map.get(key).copied().unwrap_or(0);
                            match &best {
                                None => best = Some((key.clone(), tick)),
                                Some((_, best_tick)) if tick < *best_tick => {
                                    best = Some((key.clone(), tick))
                                }
                                _ => {}
                            }
                        }
                        best.map(|(key, _)| key)
                    };

                    let Some(key) = candidate else {
                        break;
                    };

                    let removed = {
                        let mut db_write = db.write().await;
                        db_write.remove_entry(&key)
                    };

                    if let Some((stored_key, entry)) = removed {
                        let bytes = estimate_entry_bytes(&stored_key, &entry);
                        used = self.adjust_used_bytes(-(bytes as isize));
                        self.remove_key(&stored_key).await;
                    } else {
                        used = self.used_bytes.load(Ordering::Relaxed);
                    }
                }
            }
            EvictionPolicy::VolatileTtl => {
                while used > maxmemory {
                    let candidate = {
                        let mut heap_guard = heap.lock().await;
                        heap_guard.pop()
                    };

                    let Some(min) = candidate else {
                        break;
                    };

                    let should_remove = {
                        let db_read = db.read().await;
                        matches!(
                            db_read.get(&min.key),
                            Some(entry) if entry.expires_at == Some(min.expires_at)
                        )
                    };

                    if !should_remove {
                        continue;
                    }

                    let removed = {
                        let mut db_write = db.write().await;
                        db_write.remove_entry(&min.key)
                    };

                    if let Some((stored_key, entry)) = removed {
                        let bytes = estimate_entry_bytes(&stored_key, &entry);
                        used = self.adjust_used_bytes(-(bytes as isize));
                        self.remove_key(&stored_key).await;
                    }
                }
            }
        }

        Ok(used <= maxmemory)
    }
}

pub fn estimate_entry_bytes(key: &String, entry: &Entry) -> usize {
    let key_bytes = key.capacity();
    let value_bytes = value_heap_bytes(&entry.value);
    size_of::<Entry>() + size_of::<String>() + key_bytes + value_bytes
}

fn value_heap_bytes(value: &Value) -> usize {
    match value {
        Value::String(bytes) => bytes.capacity(),
        Value::List(list) => list_heap_bytes(list),
    }
}

fn list_heap_bytes(list: &VecDeque<Vec<u8>>) -> usize {
    let slots = list.capacity();
    let mut total = slots * size_of::<Vec<u8>>();
    for item in list {
        total += item.capacity();
    }
    total
}

impl LruManager {
    pub fn maxmemory(&self) -> usize {
        self.maxmemory.load(Ordering::Relaxed)
    }

    pub fn set_maxmemory(&self, value: usize) {
        self.maxmemory.store(value, Ordering::Relaxed);
    }

    pub fn policy(&self) -> EvictionPolicy {
        u8_to_policy(self.policy.load(Ordering::Relaxed))
    }

    pub fn set_policy(&self, policy: EvictionPolicy) {
        self.policy.store(policy_to_u8(policy), Ordering::Relaxed);
    }

    pub fn used_bytes(&self) -> usize {
        self.used_bytes.load(Ordering::Relaxed)
    }

    pub fn adjust_used_bytes(&self, delta: isize) -> usize {
        let mut current = self.used_bytes.load(Ordering::Relaxed);
        loop {
            let next = if delta >= 0 {
                current.saturating_add(delta as usize)
            } else {
                current.saturating_sub((-delta) as usize)
            };
            match self.used_bytes.compare_exchange(
                current,
                next,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => return next,
                Err(actual) => current = actual,
            }
        }
    }
}

fn policy_to_u8(policy: EvictionPolicy) -> u8 {
    match policy {
        EvictionPolicy::NoEviction => 0,
        EvictionPolicy::AllKeysLru => 1,
        EvictionPolicy::VolatileTtl => 2,
    }
}

fn u8_to_policy(value: u8) -> EvictionPolicy {
    match value {
        1 => EvictionPolicy::AllKeysLru,
        2 => EvictionPolicy::VolatileTtl,
        _ => EvictionPolicy::NoEviction,
    }
}
