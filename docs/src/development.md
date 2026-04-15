# Development Guide

How to extend, debug, and contribute to miniRedis.

---

## Adding a New Command

Follow the established controller pattern:

### 1. Add the Command Variant

Edit `src/model/command.rs`:

```rust
pub enum Command {
    // ... existing variants
    MYCMD { key: String, arg: String },
}
```

### 2. Parse the Command

Edit `src/parser/parse_command/mod.rs`:

```rust
"MYCMD" => {
    if resp_array.len() != 3 {
        return Err(anyhow::anyhow!("wrong number of arguments for 'mycmd' command"));
    }
    let key = expect_bulk(&resp_array, 1)?;
    let arg = expect_bulk(&resp_array, 2)?;
    Ok(Command::MYCMD { key, arg })
}
```

### 3. Create the Controller

Create `src/controllers/mycmd.rs`:

```rust
use crate::model::{DB, Command, Heap};
use crate::lru::LruManager;
use anyhow::Result;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

pub async fn mycmd_cmd(
    socket: &mut TcpStream,
    db: DB,
    heap: Heap,
    lru: LruManager,
    key: String,
    arg: String,
) -> Result<()> {
    // 1. Acquire DB lock
    let mut db = db.write().await;

    // 2. Execute logic
    // ...

    // 3. Adjust memory tracking if needed
    // lru.adjust_used_bytes(delta);

    // 4. Trigger eviction (on writes)
    // drop(db);
    // lru.evict_if_needed(&db, &heap).await;

    // 5. Write response
    socket.write_all(b"+OK\r\n").await?;
    Ok(())
}
```

### 4. Register the Controller

Edit `src/controllers/mod.rs`:

```rust
mod mycmd;
pub use mycmd::mycmd_cmd;
```

### 5. Dispatch in `handle_client.rs`

Edit the match block in `process_client()`:

```rust
match command {
    // ... existing arms
    Command::MYCMD { key, arg } => {
        mycmd_cmd(&mut socket, db, heap, lru, key, arg).await?;
    }
}
```

### 6. Add to COMMAND Metadata

Edit `src/controllers/command_cmd.rs`:

```rust
CommandInfo::new("mycmd", 3, &["write"], 1, 1, 1),
```

### 7. Update Docs

Add an entry to `docs/src/commands.md` with syntax, response, and notes.

---

## Adding a New RESP Type

If you need to extend the protocol parser:

1. Add the variant to `src/model/resp.rs`
2. Create a new parser file in `src/parser/parse_resp/`
3. Add a dispatch arm in `src/parser/parse_resp/mod.rs`
4. Add encoding helper in `src/util/resp_encode.rs` if needed

---

## Debugging

### Enable Tokio Console

Add to `Cargo.toml`:

```toml
tokio = { version = "1", features = ["full", "tracing"] }
tracing = "0.1"
tracing-subscriber = "0.3"
```

Then initialize tracing in `main.rs`:

```rust
tracing_subscriber::fmt::init();
```

### Logging Key Operations

Add tracing spans to controllers:

```rust
use tracing::{info, warn};

pub async fn set_cmd(...) -> Result<()> {
    info!(key = %key, bytes = %value.len(), "SET");
    // ...
}
```

### Inspecting RESP Traffic

For debugging protocol issues, print raw bytes in `handle_client.rs`:

```rust
if let Err(e) = parse_resp(&buf[..end]) {
    eprintln!("Parse error on buffer: {:?}", std::str::from_utf8(&buf[..end]));
    // ...
}
```

---

## Testing

The project currently has no test suite. To add tests:

### Unit Tests

Add `#[cfg(test)]` modules to individual files:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_string() {
        let buf = b"+OK\r\n";
        let resp = parse_resp(buf).unwrap().unwrap();
        assert!(matches!(resp, RESP::SimpleStrings(s) if s == "OK"));
    }
}
```

### Integration Tests

Create `tests/integration_test.rs`:

```rust
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::test]
async fn test_ping_pong() {
    // Start server in background
    // Connect client
    // Send PING
    // Assert PONG response
}
```

### Running Tests

```sh
cargo test
```

---

## Code Style

### Formatting

```sh
cargo fmt
```

The project uses `rustfmt` with default settings. The `#[rustfmt::skip]` attribute is used on `Command` enum for alignment readability.

### Linting

```sh
cargo clippy -- -D warnings
```

Fix all clippy warnings before submitting.

### Imports

- Standard library imports first (`std::...`)
- External crate imports next (`tokio::...`, `anyhow::...`)
- Crate-local imports last (`crate::...`)
- Group and sort alphabetically within groups

---

## Common Pitfalls

### Lock Deadlocks

Never hold the DB write lock while calling `evict_if_needed()` — it acquires its own lock. Drop the guard explicitly:

```rust
{
    let mut db = db.write().await;
    db.insert(...);
} // db lock dropped here
lru.evict_if_needed(&db, &heap).await;
```

### Heap Duplicates

When updating a key's TTL, always push a new heap entry. The background task verifies timestamps to filter stale entries:

```rust
if let Some(entry) = db.get_mut(&key) {
    entry.expires_at = Some(new_expiry);
    heap.push(MinHeap { expires_at: new_expiry, key: key.clone() }).await;
}
```

### Memory Leaks

Always call `lru.adjust_used_bytes()` when adding or removing data from the DB. Forgetting this causes the memory counter to drift, leading to premature or absent eviction.

### Partial RESP Parsing

Parsers return `Ok(None)` on insufficient data. Never unwrap — handle the `None` case by continuing the read loop:

```rust
match parse_resp(&buf) {
    Ok(Some(resp)) => { /* process */ }
    Ok(None) => continue, /* need more data */
    Err(e) => { /* protocol error */ }
}
```

---

## Building the Documentation

This project uses [mdBook](https://github.com/rust-lang/mdBook) for documentation.

```sh
# Install mdBook
cargo install mdbook

# Build
mdbook build docs

# Serve locally
mdbook serve docs --open
```

The book configuration is in `docs/book.toml`.
