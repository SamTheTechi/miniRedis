# Memory Management & Eviction

miniRedis provides approximate memory tracking with configurable eviction policies. This page covers the LRU manager, TTL heap, and memory accounting.

---

## Overview

```
┌───────────────────────────────────────────────────────┐
│                    LruManager                         │
│                                                       │
│  maxmemory: AtomicUsize                               │
│  used_bytes: AtomicUsize  ◄─── adjust via CAS loops   │
│  policy: AtomicU8         ◄─── runtime configurable   │
│  last_access: Mutex<HashMap<String, u64>>             │
│  access_tx: mpsc::Sender<Vec<String>>                 │
└───────────────────────┬───────────────────────────────┘
                        │
          ┌─────────────┼─────────────┐
          ▼             ▼             ▼
    ┌──────────┐  ┌──────────┐  ┌────────────┐
    │NoEviction│  │AllKeysLRU│  │VolatileTTL │
    └──────────┘  └──────────┘  └────────────┘
```

---

## Memory Tracking

### `used_bytes` (AtomicUsize)

An approximate counter for total memory usage. Updated via **compare-and-swap (CAS) loops** — no locks required:

```rust
fn adjust_used_bytes(&self, delta: isize) {
    let mut current = self.used_bytes.load(Ordering::Relaxed);
    loop {
        let new = if delta >= 0 {
            current.saturating_add(delta as usize)
        } else {
            current.saturating_sub(delta.unsigned_abs())
        };
        match self.used_bytes.compare_exchange(current, new, ...) {
            Ok(_) => break,
            Err(actual) => current = actual, // retry
        }
    }
}
```

### Memory Estimation (`estimate_entry_bytes`)

Estimates the size of a DB entry:

```
key_string.capacity()
+ size_of::<Entry>()
+ value capacity:
    String  → vec.capacity()
    List    → size_of::<VecDeque>()
            + deque.capacity() * size_of::<Option<Vec<u8>>>()
            + Σ element.capacity()
```

**Design choice:** Uses `.capacity()` instead of `.len()`. This overestimates actual data but reflects real allocation size, making eviction more accurate.

---

## Eviction Policies

### NoEviction (default)

When `used_bytes > maxmemory`:
- Returns OOM error to the client
- Controllers **roll back** mutations (restore old value or remove newly created key)

```
- OOM command not allowed when used memory > 'maxmemory'.
```

### AllKeysLru

Sample-based approximate LRU eviction across all keys.

**Algorithm:**
1. Check if `used_bytes > maxmemory`
2. Sample `SAMPLE_SIZE = 8` random keys from the DB
3. For each sampled key, look up its last access tick in the LRU map
4. Evict the key with the **lowest access tick** (least recently used)
5. Adjust `used_bytes` and repeat until under limit

**Why sample-based?** True LRU requires tracking every access with ordering overhead. Sampling 8 keys provides a good approximation with minimal cost — matching Redis's `maxmemory-samples` approach.

### VolatileTTL

Evicts keys with the soonest TTL expiration.

**Algorithm:**
1. Check if `used_bytes > maxmemory`
2. Pop from the TTL min-heap (earliest expiration first)
3. Verify the popped `expires_at` matches the DB entry's `expires_at` (handles duplicates)
4. Remove the key from DB
5. Repeat until under limit

**Duplicate handling:** When a key's TTL is updated, a new heap entry is pushed. The old entry remains with a stale timestamp. Step 3 filters these out by verifying timestamps match.

---

## LRU Access Tracking

### Batching via mpsc Channel

To avoid locking the access map on every key lookup:

```
Client Loop                    Background Task
───────────                    ───────────────
record_access("foo")    ──┐
record_access("bar")    ──┤
flush_access_batch()    ──┼──▶  mpsc::channel (cap: 1024)  ──▶  update last_access map
                          ┘
```

1. **Collection**: Each client loop maintains a `Vec<String>` buffer
2. **Recording**: `lru.record_access(key)` pushes to the buffer
3. **Flushing**: After each command, `lru.flush_access_batch()` sends the buffer through the channel
4. **Processing**: A dedicated background task receives batches and updates `last_access: HashMap<String, u64>` with an incrementing tick counter

### Access Ticks

Each key maps to a `u64` tick counter that increments on every access:

```rust
let mut tick = self.current_tick;
for key in batch {
    *self.last_access.entry(key).or_insert(tick) = tick;
    tick += 1;
}
self.current_tick = tick;
```

During LRU eviction, the key with the **lowest tick** is considered least recently used.

---

## TTL Expiration

### Dual Strategy

| Strategy | Mechanism | Frequency |
|----------|-----------|-----------|
| **Lazy** | Check on access (GET, EXISTS, TYPE, etc.) | Per-request |
| **Eager** | Background heap cleanup task | Every ~100ms |

### Lazy Expiration

When accessing a key:

```rust
if is_expired(entry) {
    // Push stale entry back to heap for background cleanup
    heap.push(entry.clone());
    return nil;
}
```

The expired key returns `nil` but stays in the DB until the background task removes it.

### Background Cleanup (`async_heap_delete.rs`)

```
loop {
    tokio::time::sleep(100ms).await;
    
    lock heap + lock db (write);
    
    while heap.peek().expires_at <= now {
        entry = heap.pop();
        if entry.expires_at == db[key].expires_at {
            db.remove(key);
            freed_bytes += estimate_entry_bytes(key);
        }
    }
    
    adjust_used_bytes(-freed_bytes);
}
```

Uses `Instant` for monotonic, precise timestamps — immune to system clock adjustments.

---

## Runtime Configuration

### CONFIG SET

```
CONFIG SET maxmemory 1048576
CONFIG SET maxmemory-policy allkeys-lru
```

Changes are applied atomically:
- `maxmemory` → `AtomicUsize::store()`
- `policy` → `AtomicU8::store()`

The LRU background task reads these values on each batch processing — no restart required.

### Environment Variables

| Variable | Purpose | Default |
|----------|---------|---------|
| `MINIREDIS_MAXMEMORY` | Byte limit (0 = disabled) | `0` |
| `MINIREDIS_MAXMEMORY_POLICY` | Eviction policy name | `noeviction` |

### CLI Flags

```
--maxmemory <bytes>
--maxmemory-policy <noeviction|allkeys-lru|volatile-ttl>
```

---

## OOM Rollback

When eviction fails under `noeviction` policy, controllers undo mutations:

### SET Rollback
```rust
let old = db.insert(key, new_entry);
if evict_if_needed().await == false {
    if let Some(old_entry) = old {
        db.insert(key, old_entry);  // restore
    } else {
        db.remove(key);              // remove new key
    }
    return OOM_ERROR;
}
```

### LPUSH/RPUSH Rollback
```rust
// If list was newly created, remove it
// If list existed, restore to previous state
```

This ensures no partial state on OOM.
