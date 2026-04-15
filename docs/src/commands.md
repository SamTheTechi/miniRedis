# Supported Commands

miniRedis supports 22 Redis-compatible commands across strings, lists, key management, and administration.

---

## String Commands

### SET

```
SET <key> <value>
```

Sets `<key>` to hold the string `<value>`. Overwrites existing values.

**Response:** `+OK\r\n`

**Memory:** Tracks byte size; triggers eviction if over `maxmemory`. On OOM failure, rolls back the insert.

---

### SETEX

```
SETEX <key> <seconds> <value>
```

Sets `<key>` with a TTL in seconds. Atomically sets value and expiry.

**Response:** `+OK\r\n`

**Internals:** Creates `Entry` with `expires_at = Instant::now() + Duration::from_secs(seconds)`, pushes to TTL min-heap.

---

### PSETEX

```
PSETEX <key> <milliseconds> <value>
```

Sets `<key>` with a TTL in milliseconds.

**Response:** `+OK\r\n`

**Internals:** Same as SETEX but uses `Duration::from_millis()`.

---

### GET

```
GET <key>
```

Returns the value of `<key>`. Performs lazy expiration on access.

**Response:** `$<len>\r\n<value>\r\n` (or `$-1\r\n` if missing/expired)

---

## List Commands

### LPUSH

```
LPUSH <key> <value> [value ...]
```

Pushes one or more values to the **head** (left) of the list. Creates a new list if the key doesn't exist.

**Response:** `:<length>\r\n` (list length after push)

**Error:** `-WRONGTYPE\r\n` if key holds a non-list value.

**Memory:** Tracks byte delta; rolls back on OOM.

---

### RPUSH

```
RPUSH <key> <value> [value ...]
```

Pushes one or more values to the **tail** (right) of the list.

**Response:** `:<length>\r\n`

**Error:** `-WRONGTYPE\r\n` if key holds a non-list value.

---

### LPOP

```
LPOP <key>
```

Removes and returns the first element from the list.

**Response:** `$<len>\r\n<value>\r\n` (or `$-1\r\n` if list is empty or key missing)

**Internals:** Removes the key entirely when the list becomes empty. Adjusts memory tracking.

---

### RPOP

```
RPOP <key>
```

Removes and returns the last element from the list.

**Response:** `$<len>\r\n<value>\r\n` (or `$-1\r\n` if list is empty or key missing)

---

## Key Management

### DEL

```
DEL <key> [key ...]
```

Removes the specified keys. A key is ignored if it does not exist.

**Response:** `:<count>\r\n` (number of keys actually removed)

**Memory:** Calculates freed bytes and adjusts the LRU tracker.

---

### EXISTS

```
EXISTS <key> [key ...]
```

Returns the number of keys that exist (non-expired).

**Response:** `:<count>\r\n`

**Internals:** Performs lazy expiration — expired keys are re-pushed to the heap for background cleanup.

---

### EXPIRE

```
EXPIRE <key> <seconds>
```

Sets a TTL on `<key>` in seconds. Overwrites any existing TTL.

**Response:** `:1\r\n` (TTL set) or `:0\r\n` (key doesn't exist)

---

### PERSIST

```
PERSIST <key>
```

Removes the TTL from `<key>`, making it persistent.

**Response:** `:1\r\n` (TTL removed) or `:0\r\n` (key doesn't exist or had no TTL)

**Internals:** Performs lazy expiration check before removing TTL.

---

### TTL

```
TTL <key>
```

Returns the remaining time to live in seconds.

**Response:**
- `:<seconds>\r\n` — remaining TTL
- `:-1\r\n` — key exists but has no TTL
- `:-2\r\n` — key doesn't exist or is expired

---

### PTTL

```
PTTL <key>
```

Returns the remaining time to live in milliseconds.

**Response:** Same format as TTL but in milliseconds.

---

### TYPE

```
TYPE <key>
```

Returns the type of value stored at `<key>`.

**Response:**
- `+string\r\n` — value is a string
- `+list\r\n` — value is a list
- `+none\r\n` — key doesn't exist

---

## Protocol & Administration

### PING

```
PING
```

Tests if the connection is alive.

**Response:** `+PONG\r\n`

---

### QUIT

```
QUIT
```

Closes the client connection.

**Response:** `+OK\r\n` (then server closes the socket)

---

### HELLO

```
HELLO [protover]
```

Protocol handshake. Accepts version 2 or 3.

**Response:** RESP2-style map with server metadata:
```
*1\r\n
*6\r\n
$4\r\n
name\r\n
$9\r\n
miniRedis\r\n
...
```

---

### COMMAND

```
COMMAND
```

Returns metadata for all 22 supported commands.

**Response:** Array of `CommandInfo` entries, each containing:
- Command name
- Arity (negative = variable args)
- Flags (`readonly`, `write`, `fast`, `admin`)
- First key position
- Last key position
- Key step

---

### INFO

```
INFO [section]
```

Returns server statistics. Supported sections: `server`, `clients`, `memory`, `stats`.

**Response:** Bulk string in Redis INFO format:
```
# Server
redis_version:0.1.0
...

# Clients
connected_clients:3
...
```

---

### CONFIG GET

```
CONFIG GET <pattern>
```

Returns configuration values matching `<pattern>`. Supports `*` wildcard.

**Response:** Array of `[key, value, key, value, ...]` pairs.

**Retrievable keys:** `maxmemory`, `maxmemory-policy`

---

### CONFIG SET

```
CONFIG SET <key> <value>
```

Sets a configuration parameter at runtime.

**Response:** `+OK\r\n`

**Settable keys:**
- `maxmemory` — byte limit (0 = disabled)
- `maxmemory-policy` — `noeviction`, `allkeys-lru`, or `volatile-ttl`

---

### CLIENT SETINFO

```
CLIENT SETINFO <attr> <value>
```

Accepted but not stored (compatibility shim for Redis clients that send this on connect).

**Response:** `+OK\r\n`

All other `CLIENT` subcommands are rejected.

---

## Command Summary Table

| Command | Arity | Type | Flags |
|---------|-------|------|-------|
| PING | 1 | fast | — |
| QUIT | 1 | fast | — |
| GET | 2 | readonly | fast |
| SET | -3 | write | — |
| SETEX | 4 | write | — |
| PSETEX | 4 | write | — |
| DEL | -2 | write | — |
| EXISTS | -2 | readonly | fast |
| EXPIRE | 3 | write | fast |
| PERSIST | 2 | write | fast |
| TTL | 2 | readonly | fast |
| PTTL | 2 | readonly | fast |
| TYPE | 2 | readonly | fast |
| LPUSH | -3 | write | — |
| RPUSH | -3 | write | — |
| LPOP | 2 | write | fast |
| RPOP | 2 | write | fast |
| CONFIG | -2 | admin, readonly | — |
| INFO | -1 | readonly | — |
| HELLO | -1 | readonly | fast |
| COMMAND | 0 | readonly | — |
| CLIENT | -2 | readonly | — |

**Arity note:** Negative values indicate variable-length argument lists. For example, `-3` means "at least 3 arguments."
