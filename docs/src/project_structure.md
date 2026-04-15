# Project Structure

A module-by-module reference for navigating the miniRedis codebase.

---

## Source Tree

```
src/
├── main.rs                      # Entry point, CLI parsing, server bootstrap
├── handle_client.rs             # Per-client TCP handling loop
├── async_heap_delete.rs         # Background TTL cleanup task
├── lru.rs                       # LRU tracking, eviction, memory accounting
│
├── model/
│   ├── mod.rs                   # Module re-exports
│   ├── db.rs                    # DB, Entry, Value types
│   ├── resp.rs                  # RESP enum (wire format types)
│   ├── command.rs               # Command enum + CommandInfo
│   └── min_heap.rs              # MinHeap (TTL min-heap wrapper)
│
├── parser/
│   ├── mod.rs                   # Module re-exports
│   ├── parse_resp/
│   │   ├── mod.rs               # RESP dispatch (first-byte routing)
│   │   ├── simple_strings.rs    # + parser
│   │   ├── simple_errors.rs     # - parser
│   │   ├── integers.rs          # : parser
│   │   ├── bulkstings.rs        # $ parser
│   │   └── arrays.rs            # * parser (recursive)
│   └── parse_command/
│       └── mod.rs               # RESP array → Command enum
│
├── controllers/
│   ├── mod.rs                   # Module re-exports
│   ├── get.rs                   # GET
│   ├── set.rs                   # SET
│   ├── setex.rs                 # SETEX
│   ├── psetex.rs                # PSETEX
│   ├── del.rs                   # DEL
│   ├── exists.rs                # EXISTS
│   ├── expire.rs                # EXPIRE
│   ├── persist.rs               # PERSIST
│   ├── ttl.rs                   # TTL
│   ├── pttl.rs                  # PTTL
│   ├── type_cmd.rs              # TYPE
│   ├── info.rs                  # INFO
│   ├── config.rs                # CONFIG GET / CONFIG SET
│   ├── hello.rs                 # HELLO
│   ├── command_cmd.rs           # COMMAND
│   ├── lpush.rs                 # LPUSH
│   ├── rpush.rs                 # RPUSH
│   ├── lpop.rs                  # LPOP
│   └── rpop.rs                  # RPOP
│
└── util/
    ├── mod.rs                   # Module re-exports
    ├── bulk_to_string.rs        # Vec<u8> → String helper
    ├── expect_bulk.rs           # Validate/extract bulk string at index
    ├── find_crlf.rs             # Find \r\n in byte slice
    ├── is_expired.rs            # Check if Entry has expired
    └── resp_encode.rs           # RESP serialization helpers
```

---

## Module Details

### `main.rs`

**Purpose:** Server bootstrap and accept loop.

**Key responsibilities:**
- Parse CLI args and env vars
- Create `TcpListener`
- Initialize shared state (`DB`, `Heap`, `LruManager`)
- Launch background cleanup task
- Accept connections and spawn per-client tasks

---

### `handle_client.rs`

**Purpose:** Main loop for a single client connection.

**Key responsibilities:**
- Read bytes from socket (up to 4096 per read)
- Validate RESP first byte
- Parse RESP messages incrementally
- Convert to `Command` enum
- Record LRU access
- Dispatch to controller
- Write RESP response
- Flush access batch

---

### `async_heap_delete.rs`

**Purpose:** Periodic TTL cleanup.

**Key responsibilities:**
- Sleep 100ms between cycles
- Pop expired entries from heap
- Remove from DB
- Adjust memory counter

---

### `lru.rs`

**Purpose:** Memory tracking and eviction.

**Key types:**
- `LruManager` — holds counters, access map, and channel sender
- `EvictionPolicy` — enum: `NoEviction`, `AllKeysLru`, `VolatileTtl`

**Key functions:**
- `new(maxmemory, policy)` — create manager + spawn background access task
- `record_access(key)` — buffer a key access
- `flush_access_batch()` — send buffered keys through channel
- `evict_if_needed()` — check limit and evict per policy
- `adjust_used_bytes(delta)` — CAS-loop atomic update
- `estimate_entry_bytes(key, db)` — approximate entry size

---

### `model/db.rs`

**Key types:**

```rust
pub enum Value {
    String(Vec<u8>),
    List(VecDeque<Vec<u8>>),
}

pub struct Entry {
    pub value: Value,
    pub expires_at: Option<Instant>,
}

pub type DB = Arc<RwLock<HashMap<String, Entry>>>;
```

**Key methods:**
- `Value::to_resp_bytes()` — serialize to RESP
- `Value::as_list_mut()` — downcast to mutable VecDeque

---

### `model/resp.rs`

```rust
pub enum RESP {
    SimpleStrings(String),
    SimpleErrors(String),
    Integers(i64),
    BulkStrings(Option<Vec<u8>>),  // None = nil
    Arrays(Vec<RESP>),
}
```

---

### `model/command.rs`

**Key types:**
- `Command` — enum with 22 variants, each carrying typed arguments
- `CommandInfo` — metadata for the COMMAND response (name, arity, flags, key positions)

---

### `model/min_heap.rs`

```rust
pub struct MinHeap {
    pub expires_at: Instant,
    pub key: String,
}

pub type Heap = Arc<Mutex<BinaryHeap<MinHeap>>>;
```

**Key detail:** `Ord` is reversed (`other.expires_at.cmp(&self.expires_at)`) so Rust's max-heap behaves as a min-heap — earliest expiration bubbles to the top.

---

### `parser/parse_resp/`

**Entry point:** `parse_resp(buf: &[u8]) -> Result<Option<RESP>>`

Dispatches by first byte:
- `+` → `simple_strings.rs`
- `-` → `simple_errors.rs`
- `:` → `integers.rs`
- `$` → `bulkstings.rs`
- `*` → `arrays.rs` (recursively calls `parse_resp` for each element)

All return `Ok(None)` on insufficient data.

---

### `parser/parse_command/`

**Entry point:** `parse_command(resp_array: Vec<RESP>) -> Result<Command>`

Matches first element (command name) as string, then validates argument count and extracts typed fields using `expect_bulk()`.

---

### `controllers/`

One file per command. Common pattern:

```rust
pub async fn <cmd>_cmd(socket: &mut TcpStream, db: DB, heap: Heap, lru: LruManager) -> Result<()> {
    // 1. Acquire lock
    // 2. Validate / check expiration
    // 3. Mutate or read
    // 4. Adjust memory tracking
    // 5. Trigger eviction (on writes)
    // 6. Rollback on OOM
    // 7. Serialize response
    // 8. Write to socket
}
```

---

### `util/`

| Function | Purpose |
|----------|---------|
| `find_crlf(buf)` | Locate `\r\n` boundary |
| `bulk_to_string(bytes)` | `Vec<u8>` → `String` (lossy UTF-8) |
| `expect_bulk(array, index)` | Validate element at index is a bulk string and extract it |
| `is_expired(entry)` | Check `entry.expires_at <= Instant::now()` |
| `array_len(n)` | Serialize `*<n>\r\n` |
| `bulk_str(s)` | Serialize `$<len>\r\n<data>\r\n` |
| `integer(n)` | Serialize `:<n>\r\n` |

---

## Type Aliases

| Alias | Resolves To |
|-------|-------------|
| `DB` | `Arc<RwLock<HashMap<String, Entry>>>` |
| `Heap` | `Arc<Mutex<BinaryHeap<MinHeap>>>` |

Defined in `model/mod.rs` and re-exported at the crate root.
