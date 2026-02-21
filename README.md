# miniRedis

miniRedis is a simplified, in-memory, asynchronous Redis-like server built with Rust and Tokio. It's a learning project that demonstrates how to build a high-performance TCP server that can handle multiple clients concurrently, parse the RESP (REdis Serialization Protocol), and execute a subset of Redis commands.

## Overview

The server listens for incoming TCP connections on `127.0.0.1:6379`. For each connection, it spawns a new asynchronous task to handle client communication. The server uses a shared, thread-safe, in-memory `HashMap` to store data.

The server's core logic is as follows:
1.  Read data from the client.
2.  Parse the data into a RESP message.
3.  Parse the RESP message into a command.
4.  Execute the command.
5.  Write the response back to the client.

## Features

The following commands are supported:

-   `PING`: Returns `PONG`. Used to test if the connection is still alive.
-   `GET <key>`: Returns the value of `<key>`. If the key does not exist, `nil` is returned.
-   `SET <key> <value>`: Sets `<key>` to hold the string `<value>`.
-   `SETEX <key> <seconds> <value>`: Set value and expire after seconds.
-   `PSETEX <key> <milliseconds> <value>`: Set value and expire after milliseconds.
-   `DEL <key> [key ...]` : Removes the specified keys. A key is ignored if it does not exist.
-   `EXISTS <key> [key ...]` : Returns the number of `<key>`s that exist.
-   `EXPIRE <key> <seconds>`: Set a key's time to live.
-   `PERSIST <key>`: Remove a key's TTL.
-   `TTL <key>` / `PTTL <key>`: Return remaining time to live.
-   `TYPE <key>`: Return key type.
-   `LPUSH <key> <value ...>` / `RPUSH <key> <value ...>`: Push values to a list.
-   `LPOP <key>` / `RPOP <key>`: Pop values from a list.
-   `CONFIG GET/SET`: Runtime configuration for `maxmemory` and `maxmemory-policy`.
-   `INFO`: Basic server stats.
-   `QUIT`: Close the connection.

Eviction and memory limits:

-   `maxmemory` (approximate) with `maxmemory-policy`:
    -   `noeviction`
    -   `allkeys-lru` (approximate)
    -   `volatile-ttl`

Protocol support:

-   RESP only (inline protocol is not supported)

## RESP Implementation

The server implements a parser for the following RESP data types:
-   Simple Strings (`+`)
-   Simple Errors (`-`)
-   Integers (`:`)
-   Bulk Strings (`$`)
-   Arrays (`*`)

## Getting Started

### Prerequisites

-   Rust and Cargo
-   A Redis client, like `redis-cli`

### Running the server

1.  Clone the repository:
    ```sh
    git clone <repository-url>
    cd miniRedis
    ```
2.  Build and run the server:
    ```sh
    cargo run
    ```
The server will start and listen on `127.0.0.1:6379`.

### Command-line options

When using `cargo run`, arguments to the server must come after `--`:

```sh
cargo run -- -h
```

If you run the compiled binary directly, no `--` is needed:

```sh
./target/debug/miniRedis -h
```

Supported options:

- `--bind <ip>`: bind address (default `127.0.0.1`)
- `--port <port>`: port (default `6379`)
- `--maxmemory <bytes>`: approximate max memory (default `0`, disabled)
- `--maxmemory-policy <noeviction|allkeys-lru|volatile-ttl>`
- `-h` / `--help`: show help

## Usage

You can interact with the server using `redis-cli` in a separate terminal:

```sh
# Set a key
$ redis-cli SET mykey "Hello, world!"
OK

# Get a key
$ redis-cli GET mykey
"Hello, world!"

# Check if a key exists
$ redis-cli EXISTS mykey
(integer) 1

# Delete a key
$ redis-cli DEL mykey
(integer) 1

# Check if the key still exists
$ redis-cli GET mykey
(nil)
```

## Project Structure

```
├── src
│   ├── main.rs            # Entry point, sets up the TCP listener and shared state
│   ├── handle_client.rs   # Main loop for handling a client connection
│   ├── model
│   │   ├── db.rs          # DB types and values
│   │   ├── command.rs     # Command enum
│   │   ├── resp.rs        # RESP enum
│   │   └── min_heap.rs    # TTL min-heap
│   ├── parser
│   │   ├── mod.rs         # Exports the parser modules
│   │   ├── parse_resp     # Low-level RESP parsing functions
│   │   └── parse_command  # High-level command parsing from RESP Arrays
│   │   └── parse_inline   # Inline command parsing
│   ├── controllers
│   │   ├── mod.rs         # Exports the command controller modules
│   │   ├── get.rs         # GET command implementation
│   │   ├── set.rs         # SET command implementation
│   │   ├── del.rs         # DEL command implementation
│   │   └── exists.rs      # EXISTS command implementation
│   │   ├── config.rs      # CONFIG GET/SET
│   │   ├── info.rs        # INFO
│   │   ├── persist.rs     # PERSIST
│   │   └── type_cmd.rs    # TYPE
│   ├── lru.rs             # Approximate LRU + maxmemory eviction
│   └── util               # Utility functions
├── Cargo.toml             # Project dependencies and metadata
└── README.md              # This file
```
