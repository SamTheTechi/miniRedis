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
-   `DEL <key> [key ...]` : Removes the specified keys. A key is ignored if it does not exist.
-   `EXISTS <key> [key ...]` : Returns the number of `<key>`s that exist.

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
│   │   └── types.rs       # Core data structures (DB, Command, RESP)
│   ├── parser
│   │   ├── mod.rs         # Exports the parser modules
│   │   ├── parse_resp     # Low-level RESP parsing functions
│   │   └── parse_command  # High-level command parsing from RESP Arrays
│   ├── controllers
│   │   ├── mod.rs         # Exports the command controller modules
│   │   ├── get.rs         # GET command implementation
│   │   ├── set.rs         # SET command implementation
│   │   ├── del.rs         # DEL command implementation
│   │   └── exists.rs      # EXISTS command implementation
│   └── util               # Utility functions
├── Cargo.toml             # Project dependencies and metadata
└── README.md              # This file
```
