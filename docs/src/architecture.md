# Architecture

miniRedis follows a layered, async-first architecture built on Tokio's runtime. This page covers the overall system design, concurrency model, and request flow.

---

## System Overview

```
┌─────────────┐     TCP (RESP)     ┌──────────────────────────────┐
│  redis-cli   │ ──────────────────▶│        miniRedis Server      │
│  or any      │ ◀───────────────── │                              │
│  RESP client │                    │  ┌────────────────────────┐  │
└─────────────┘                    │  │  TcpListener (:6379)    │  │
                                   │  └──────────┬─────────────┘  │
                                   │             │ accept         │
                                   │             ▼                │
                                   │  ┌────────────────────────┐  │
                                   │  │  tokio::spawn          │  │
                                   │  │  process_client()      │  │
                                   │  └──────────┬─────────────┘  │
                                   │             │ dispatch       │
                                   │   ┌─────────┼──────────┐     │
                                   │   ▼         ▼          ▼     │
                                   │ ┌────┐  ┌──────┐  ┌───────┐  │
                                   │ │GET │  │SET   │  │LPUSH  │  │
                                   │ └──┬─┘  └──┬───┘  └───┬───┘  │
                                   │    │       │          │      │
                                   │    └───────┼──────────┘      │
                                   │            ▼                 │
                                   │  ┌────────────────────────┐  │
                                   │  │  DB (HashMap + RWLock) │  │
                                   │  │  TTL Heap (MinHeap)    │  │
                                   │  │  LRU Manager           │  │
                                   │  └────────────────────────┘  │
                                   └──────────────────────────────┘
```

---

## Core Components

### Entry Point (`main.rs`)

The server bootstrap performs:

1. **CLI argument parsing** — `--bind`, `--port`, `--maxmemory`, `--maxmemory-policy`
2. **Environment variable fallback** — `MINIREDIS_MAXMEMORY`, `MINIREDIS_MAXMEMORY_POLICY`
3. **Shared state initialization**:
   - `DB` — `Arc<RwLock<HashMap<String, Entry>>>` for key-value storage
   - `Heap` — `Arc<Mutex<BinaryHeap<MinHeap>>>` for TTL expiration tracking
   - `LruManager` — approximate LRU tracking and memory accounting
4. **Background task launch** — `async_clean_db_heap` spawns a periodic TTL cleanup task
5. **TCP accept loop** — each connection spawns a dedicated `tokio::spawn` task

### Client Handler (`handle_client.rs`)

`process_client()` is the per-client async loop:

1. Reads up to 4096 bytes into a ring buffer
2. Validates the first byte is a valid RESP type (`+`, `-`, `:`, `$`, `*`)
3. Parses RESP messages incrementally (returns `Ok(None)` on partial data)
4. Converts RESP arrays into `Command` enum variants
5. Records key access for LRU tracking before dispatch
6. Dispatches to the appropriate controller
7. Flushes the access batch after each command
8. Writes RESP response back to the socket

### Background Cleanup (`async_heap_delete.rs`)

A dedicated tokio task runs every 100ms:

1. Locks the heap (mutex) and DB (write lock)
2. Pops entries where `expires_at <= Instant::now()`
3. Removes expired keys from the DB
4. Calculates freed bytes and adjusts the LRU memory tracker
5. Uses `Instant` for precise, monotonic timestamps

---

## Concurrency Model

### Shared State

| Component | Type | Purpose |
|-----------|------|---------|
| DB | `Arc<RwLock<HashMap>>` | Concurrent reads, exclusive writes |
| TTL Heap | `Arc<Mutex<BinaryHeap>>` | Exclusive access only |
| LRU Manager | `Arc<AtomicU*>` + `mpsc::Sender` | Lock-free counters, batched channel |

### Lock Ordering

The code avoids deadlocks by dropping guards before acquiring other locks. For example, in `set_cmd`:

```rust
{
    let mut db = db.write().await;
    // ... perform insert ...
    db.drop(); // explicit drop before eviction
}
evict_if_needed(...).await; // acquires its own DB lock
```

### LRU Access Batching

To avoid locking the LRU map on every key access:

1. Each client loop collects key accesses into a `Vec<String>` buffer
2. After each command, the buffer is flushed via an `mpsc::channel` (capacity 1024)
3. A dedicated background task receives batches and updates the `last_access` map
4. Batch size is 32 accesses per flush

---

## Data Types

### `Value` (`model/db.rs`)

```rust
pub enum Value {
    String(Vec<u8>),
    List(VecDeque<Vec<u8>>),
}
```

### `Entry` (`model/db.rs`)

```rust
pub struct Entry {
    pub value: Value,
    pub expires_at: Option<Instant>,
}
```

### `Command` (`model/command.rs`)

An enum with 22 variants covering all supported Redis commands. Each variant carries its typed arguments:

```rust
pub enum Command {
    PING,
    QUIT,
    SET { key: String, value: Vec<u8> },
    SETEX { key: String, value: Vec<u8>, seconds: u64 },
    GET { key: String },
    DEL { keys: Vec<String> },
    // ... etc
}
```

---

## Request Lifecycle

```
1. TCP bytes arrive
2. Buffer accumulates (partial read handling)
3. find_crlf() locates \r\n delimiters
4. parse_resp() → RESP enum (recursive descent)
5. parse_command() → Command enum (type-safe dispatch)
6. Controller executes (acquires DB lock as needed)
7. Response serialized as RESP bytes
8. Bytes written to socket
9. LRU access batch flushed
10. Loop back to step 1
```
