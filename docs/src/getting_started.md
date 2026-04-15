# Getting Started

How to build, run, and connect to miniRedis.

---

## Prerequisites

- **Rust & Cargo** — install via [rustup](https://rustup.rs/)
- **A RESP client** — `redis-cli` is recommended but any RESP-compatible client works

---

## Building

```sh
cargo build
```

For an optimized release build:

```sh
cargo build --release
```

---

## Running

### Default

```sh
cargo run
```

The server starts on `127.0.0.1:6379`.

### With Options

```sh
cargo run -- --bind 0.0.0.0 --port 6380 --maxmemory 1048576 --maxmemory-policy allkeys-lru
```

Or run the binary directly:

```sh
./target/debug/miniRedis --help
./target/debug/miniRedis --bind 0.0.0.0 --port 6380
```

### CLI Flags

| Flag | Description | Default |
|------|-------------|---------|
| `--bind <ip>` | Bind address | `127.0.0.1` |
| `--port <port>` | Port number | `6379` |
| `--maxmemory <bytes>` | Approximate memory limit (0 = disabled) | `0` |
| `--maxmemory-policy <policy>` | Eviction policy | `noeviction` |
| `--help`, `-h` | Show help and exit | — |

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `MINIREDIS_MAXMEMORY` | Byte limit (overridden by CLI flag if both set) | `0` |
| `MINIREDIS_MAXMEMORY_POLICY` | Eviction policy name | `noeviction` |

---

## Connecting with redis-cli

```sh
# Basic connection
redis-cli -h 127.0.0.1 -p 6379

# Test connectivity
redis-cli PING
# → PONG
```

### Example Session

```sh
# Set a string
redis-cli SET greeting "Hello, miniRedis!"
# → OK

# Get it back
redis-cli GET greeting
# → "Hello, miniRedis!"

# Set with TTL
redis-cli SETEX temp_key 10 "expires soon"
# → OK

# Check TTL
redis-cli TTL temp_key
# → (integer) 8

# List operations
redis-cli LPUSH mylist a b c
# → (integer) 3

redis-cli LPOP mylist
# → "c"

# Delete
redis-cli DEL greeting temp_key
# → (integer) 2

# Server info
redis-cli INFO memory
# → # Memory
# → used_memory:1234
# → ...
```

---

## Configuration at Runtime

Use `CONFIG GET` and `CONFIG SET` after the server is running:

```sh
# Check current settings
redis-cli CONFIG GET maxmemory
# → 1) "maxmemory"
# → 2) "0"

redis-cli CONFIG GET maxmemory-policy
# → 1) "maxmemory-policy"
# → 2) "noeviction"

# Change memory limit
redis-cli CONFIG SET maxmemory 2097152
# → OK

# Change eviction policy
redis-cli CONFIG SET maxmemory-policy allkeys-lru
# → OK
```

---

## Supported vs Unsupported Features

### Supported

- String GET/SET with TTL
- List push/pop (LPUSH, RPUSH, LPOP, RPOP)
- Key management (DEL, EXISTS, EXPIRE, PERSIST, TTL, PTTL, TYPE)
- Approximate LRU and TTL-based eviction
- Runtime configuration via CONFIG
- COMMAND metadata
- INFO sections (server, clients, memory, stats)
- HELLO handshake (v2/v3)

### Not Implemented

- Hashes, Sets, Sorted Sets, Bitmaps, HyperLogLog, Streams
- Transactions (MULTI/EXEC/DISCARD)
- Pub/Sub
- Lua scripting
- Persistence (RDB snapshots, AOF)
- Replication / clustering
- RESP3 protocol
- Inline commands (plain text)
