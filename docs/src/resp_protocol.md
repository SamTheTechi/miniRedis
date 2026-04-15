# RESP Protocol Implementation

miniRedis implements the **RESP (REdis Serialization Protocol)** for client-server communication. This page documents the protocol support, parsing strategy, and serialization.

---

## Overview

RESP is a typed, line-oriented protocol where each message begins with a type prefix character followed by `\r\n`-terminated data. miniRedis supports **RESP clients only** — inline commands (plain text like `GET foo`) are explicitly rejected.

### Type Prefixes

| Prefix | Type | Example |
|--------|------|---------|
| `+` | Simple String | `+OK\r\n` |
| `-` | Simple Error | `-ERR unknown command\r\n` |
| `:` | Integer | `:1\r\n` |
| `$` | Bulk String | `$5\r\nhello\r\n` |
| `*` | Array | `*2\r\n$3\r\nGET\r\n$3\r\nfoo\r\n` |

---

## RESP Parsing

### Parser Architecture

The parser is split into two layers:

**Layer 1 — Wire Format** (`parser/parse_resp/`):  
Recursive descent parser that dispatches by first byte to the appropriate type parser.

**Layer 2 — Command Parsing** (`parser/parse_command/`):  
Maps a `Vec<RESP>` (array) into a typed `Command` enum.

### Incremental Parsing

All RESP parsers return `Result<Option<RESP>>`:

- `Ok(Some(resp))` — complete message parsed
- `Ok(None)` — insufficient data, need more bytes
- `Err(...)` — protocol error

This enables **partial read handling**: if a TCP read returns incomplete data, the parser simply returns `None` and the client loop accumulates more bytes.

### Finding Delimiters (`util/find_crlf.rs`)

```rust
pub fn find_crlf(buf: &[u8]) -> Option<usize> {
    buf.windows(2).position(|w| w == b"\r\n")
}
```

Used by every parser to locate `\r\n` boundaries.

### Parser Modules

| File | Handles | Format |
|------|---------|--------|
| `simple_strings.rs` | `+` | `+<string>\r\n` |
| `simple_errors.rs` | `-` | `-<string>\r\n` |
| `integers.rs` | `:` | `:<number>\r\n` |
| `bulkstings.rs` | `$` | `$<len>\r\n<data>\r\n` or `$-1\r\n` (nil) |
| `arrays.rs` | `*` | `*<count>\r\n` followed by N RESP elements |

### Bulk String Nil Handling

`$-1\r\n` represents a nil bulk string. The parser produces `RESP::BulkStrings(None)`, which controllers interpret as a missing key or null value.

---

## Command Parsing (`parser/parse_command/`)

Takes a parsed `RESP::Arrays(Vec<RESP>)` and matches on the first element (the command name):

```
["SET", "foo", "bar"]  →  Command::SET { key: "foo", value: [104, 97, 114] }
["GET", "foo"]         →  Command::GET { key: "foo" }
["DEL", "a", "b"]      →  Command::DEL { keys: ["a", "b"] }
```

### Validation

- **Argument count**: Each command validates its minimum/maximum arity
- **Type checking**: Arguments must be bulk strings; wrong types produce parse errors
- **Subcommand routing**: `CONFIG GET`, `CONFIG SET`, `CLIENT SETINFO` are routed by matching subsequent array elements

### Error Messages

Parse errors follow Redis conventions:

```
-wrong number of arguments for 'get' command
-unknown command
-expected bulk string at index 1
```

---

## Response Serialization

### RESP Encoding (`util/resp_encode.rs`)

Three helper functions serialize values back to RESP:

```rust
pub fn array_len(len: usize) -> Vec<u8>   // *<count>\r\n
pub fn bulk_str(s: &str) -> Vec<u8>        // $<len>\r\n<data>\r\n
pub fn integer(n: i64) -> Vec<u8>          // :<n>\r\n
```

### Response Patterns

| Scenario | Response |
|----------|----------|
| Success (no data) | `+OK\r\n` |
| Success with value | `$<len>\r\n<value>\r\n` |
| Key not found | `$-1\r\n` |
| Integer result | `:<n>\r\n` |
| Error | `-ERR <message>\r\n` |
| Array result | `*<count>\r\n...` |
| Empty list | `*0\r\n` |

### Value Serialization (`model/db.rs`)

`Value::to_resp_bytes()` converts stored values to RESP:

- `String(bytes)` → bulk string
- `List(deque)` → array of bulk strings (empty list → `*0\r\n`)

---

## Protocol Limitations

| Feature | Status |
|---------|--------|
| RESP2 | ✅ Full support |
| RESP3 | ❌ Not supported |
| Inline commands | ❌ Rejected with protocol error |
| Pipelining | ✅ Works (messages parsed sequentially from buffer) |
| Pub/Sub | ❌ Not implemented |
| Transactions (MULTI/EXEC) | ❌ Not implemented |
| Lua scripting | ❌ Not implemented |

---

## Wire Example

### SET Command

**Client sends:**
```
*3\r\n
$3\r\n
SET\r\n
$3\r\n
foo\r\n
$3\r\n
bar\r\n
```

**Server responds:**
```
+OK\r\n
```

### GET Command

**Client sends:**
```
*2\r\n
$3\r\n
GET\r\n
$3\r\n
foo\r\n
```

**Server responds (key exists):**
```
$3\r\n
bar\r\n
```

**Server responds (key missing):**
```
$-1\r\n
```
