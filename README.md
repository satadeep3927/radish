# Radish: Lightweight, Local-First Valkey-Compatible Database & Studio

<p align="center">
  <strong>A native, zero-overhead development database, admin tool, CLI client, and visual studio for local Valkey & Redis workflows—all packed into a single binary.</strong>
</p>

---

## Table of Contents
1. [The Story: Why Valkey on Windows is a Pain](#the-story-why-valkey-on-windows-is-a-pain)
2. [What Radish Tries to Do](#what-radish-tries-to-do)
3. [What Radish Does NOT Try to Do](#what-radish-does-not-try-to-do)
4. [Architecture Overview](#architecture-overview)
5. [Performance Benchmarks](#performance-benchmarks)
6. [Configuration & File Layout](#configuration--file-layout)
7. [Supported Valkey/Redis Commands](#supported-valkeyredis-commands)
8. [Getting Started & Development](#getting-started--development)
9. [Unified CLI Reference & Admin Guide](#unified-cli-reference--admin-guide)

---

## Performance Benchmarks
Radish uses a multi-threaded, lockless-reader architecture (`RwLock<im::HashMap>`) that allows it to scale vertically across multiple CPU cores. 

**Environment:** Windows Loopback (TCP)
**Command:** `./redis-benchmark -h 127.0.0.1 -p 6379 -c 100 -t set,get -n 1000000 -q -P 200`

- **GET (Reads):** ~2,500,000 requests per second (Concurrent Lock-Free Reads)
- **SET (Writes):** ~722,000 requests per second (Exclusive Write Lock with O(1) cloning)

This bypasses the classic single-threaded Redis bottleneck while preserving O(1) background snapshotting via software-level Copy-On-Write.

---

## The Story: Why Valkey on Windows is a Pain

Valkey (and its predecessor Redis) is the backbone of modern web applications, serving as a high-performance cache, session store, and message broker. However, for developers on Windows, setting up a local Valkey/Redis instance for development has historically been a massive headache:

* **No Official Native Support**: Valkey and Redis are designed natively for Unix-like operating systems. Official binaries do not exist for Windows.
* **WSL2 Overhead**: Developers are forced to run instances in WSL2 (Windows Subsystem for Linux). While powerful, WSL2 introduces virtual machine startup overhead, consumes significant RAM, and complicates IP/port routing (such as connecting from a native Windows host to a WSL2 localhost loopback).
* **Docker Bloat**: Running Redis or Valkey via Docker Desktop on Windows requires WSL2 backends, eating up gigabytes of memory just to run a simple, lightweight cache service.
* **Obsolete Native Ports**: The older native Windows ports (like the MSOpenTech Redis 3.x forks) are ancient, unsupported, and lack modern commands, security patches, and protocol extensions.

**Radish** was built to solve this. It provides a native, zero-dependency, ultra-lightweight Valkey-compatible server and GUI studio that runs natively on Windows with the speed and footprint of a native executable.

---

## What Radish Tries to Do

Radish aims to be the ultimate local-first developer companion for Valkey/Redis workflows:

* **Native Windows Speed**: Zero Docker, zero WSL2, zero VM overhead. It compiles to a native Windows binary that starts instantly and consumes minimal resources.
* **RESP Protocol Compatibility**: Implements a highly optimized RESP (REdis Serialization Protocol) engine that works out of the box with standard Redis/Valkey CLI tools, client libraries (`redis-py`, `node-redis`, `go-redis`, `redis-rs`), and frameworks.
* **Embedded Visual Management (Studio)**: Houses a sleek, modern desktop interface powered by Tauri and SolidJS to let you inspect keyspaces, publish/subscribe to channels, manage configurations, and run interactive command terminals visually.
* **O(1) Performance Caching**: Implements thread-safe atomic keyspace size tracking, enabling real-time database sizing queries (`INFO memory`) to return in $O(1)$ constant time.
* **Safer Concurrency**: Built entirely in Rust, guaranteeing memory safety, thread-safe asynchronous networking, and non-blocking I/O.
* **Persistent Configuration**: Maintains a clean, human-readable config path (`~/.radish/radish.toml`) and database snapshot (`~/.radish/dump.radish`) situated in the user's home folder.

---

## What Radish Does NOT Try to Do

Radish is designed as a **development and testing tool**. It does **not** aim to replace production-grade databases:

* **No Clustering or Sentinel**: Does not support cluster commands, partitioning, primary-replica replication, or Sentinel high-availability loops.
* **No Lua Scripting Engine**: Does not embed full Lua execution runtimes (eval/script commands).
* **No Advanced Module Systems**: Does not support compiling or running native Redis/Valkey module extensions.
* **Not an Enterprise Production Store**: While highly performant in Rust, it does not prioritize disk write optimizations (like append-only files with fsync tuning) or enterprise-scale network concurrent threading models needed to handle millions of active clients.

---

## Architecture Overview

Radish is structured as a cargo workspace containing four modular Rust crates, wrapped in a Tauri desktop shell:

```
radish/
├── src/                      # SolidJS + TypeScript Frontend
├── src-tauri/                # Tauri Core Shell & Bridge Commands
│   └── crates/
│       ├── radish-proto/     # RESP protocol parsing crate
│       ├── radish-storage/   # In-memory keyspace & snapshoting engine
│       ├── radish-server/    # TCP command routing & connection manager
│       └── radish-cli/       # CLI repl binary target
```

* **`radish-proto`**: Parses and serializes RESP structures (Simple Strings, Errors, Integers, Bulk Strings, Arrays, and Null frames).
* **`radish-storage`**: Implements the `Keyspace` structure using structural-sharing snapshots for isolated database dumps and atomic calculations for O(1) size queries.
* **`radish-server`**: Implements the network stack, tcp client loops, pub/sub channels, commands dispatching, and background persistence ticking.
* **`radish-cli`**: A standard terminal REPL interface to query Radish or any remote RESP server.
* **Tauri Studio**: The SolidJS GUI client that communicates with the local engine process and Tauri bridge commands.

---

## Configuration & File Layout

Radish stores its configurations and data safely in the user's home folder directory under `~/.radish/` (`%USERPROFILE%\.radish\` on Windows, `~/.radish/` on macOS/Linux).

### Configuration (`~/.radish/radish.toml`)
```toml
port = 6379
bind = "127.0.0.1"
requires_auth = false
password = ""
dump_path = "dump.radish"
save_interval = 60
maxmemory = "0"
```

* `port`: The port the database listener binds to.
* `bind`: The local interface IP.
* `requires_auth`: Set to `true` to require authentication.
* `password`: The server authentication password.
* `dump_path`: The name of the snapshot file (saved relative to `~/.radish/` if not absolute).
* `save_interval`: Seconds between database auto-saves (0 or empty to disable).
* `maxmemory`: Memory limits (e.g. `0` for unlimited, `256mb`, `1gb`).

---

## Supported Valkey/Redis Commands

Radish implements a comprehensive developer command subset:

* **Generic**: `PING`, `ECHO`, `SELECT`, `AUTH`, `SHUTDOWN`, `KEYS`, `SCAN`, `FLUSHDB`, `FLUSHALL`, `DEL`, `EXISTS`, `RENAME`, `TYPE`, `EXPIRE`, `TTL`, `PEXPIRE`, `PTTL`, `EXPIREAT`, `PEXPIREAT`, `PERSIST`, `OBJECT ENCODING`
* **Strings**: `GET`, `SET` (supports `EX/PX/NX/XX`), `SETEX`, `SETNX`, `GETSET`, `MGET`, `MSET`, `INCR`, `DECR`, `GETRANGE`
* **Lists**: `LPUSH`, `RPUSH`, `LPOP`, `RPOP`, `LLEN`, `LRANGE`
* **Hashes**: `HSET`, `HGET`, `HDEL`, `HEXISTS`, `HKEYS`, `HVALS`, `HGETALL`
* **Pub/Sub**: `PUBLISH`, `SUBSCRIBE`, `UNSUBSCRIBE`, `PSUBSCRIBE`, `PUNSUBSCRIBE`
* **Security & Memory**: `ACL WHOAMI`, `ACL SETUSER`, `MEMORY USAGE`

---

## Getting Started & Development

### Prerequisites
* [Node.js](https://nodejs.org/) (v18+)
* [Rust & Cargo](https://rustup.rs/) (1.75+)

### Installation & Run

1. Clone the repository and install npm packages:
   ```bash
   npm install
   ```

2. Run the Tauri Desktop application in development mode:
   ```bash
   npm run tauri dev
   ```

3. Run the Rust test suites:
   ```bash
   cd src-tauri
   cargo test --workspace
   ```

4. Build the production application package:
   ```bash
   npm run tauri build
   ```

---

## Unified CLI Reference & Admin Guide

Radish ships as a single, multi-tool executable. If run without arguments, it launches the graphical Radish Studio. When supplied with a subcommand argument, it functions as a highly customizable CLI administrator tool or command-line client.

### Subcommand Reference

```bash
radish [start | cli | status | stop | flush | inspect | service | studio] [options]
```

---

### 1. `radish start`
Starts the native, in-memory Valkey-compatible database server engine directly in your terminal.

* **Usage**:
  ```bash
  radish start [--port <port_number>] [--save <interval_seconds>]
  ```
* **Options**:
  * `--port <port_number>`: Overrides the default or `radish.toml` configuration port (e.g. `6380`).
  * `--save <interval_seconds>`: Overrides the database auto-save tick frequency (set to `0` to disable automatic snapshots).
* **Examples**:
  ```bash
  # Starts server with home directory configuration settings
  radish start
  
  # Starts server listening on port 6385, snapshotting every 30 seconds
  radish start --port 6385 --save 30
  ```

---

### 2. `radish cli`
Starts an interactive terminal REPL connection to a Radish database engine or any standard, remote Valkey/Redis instance.

* **Usage**:
  ```bash
  radish cli [--host <host_address>] [--port <port_number>]
  ```
* **Options**:
  * `--host <host_address>`: The IP address of the target server (defaults to `127.0.0.1`).
  * `--port <port_number>`: The TCP port of the target server (defaults to `6379`).
* **Interactive Shell Features**:
  * Persistent command history navigation (using Up/Down arrow keys).
  * Auto-completion hints for all supported RESP database commands.
  * Fully syntax-highlighted responses (Integers in blue, Errors in red, OK responses in green).
* **Examples**:
  ```bash
  # Connect to the default local database instance
  radish cli
  
  # Connect to a remote server running on port 7001
  radish cli --host 192.168.1.50 --port 7001
  ```

---

### 3. `radish status`
Checks the online status of a local Radish server instance by issuing a low-level PING probe.

* **Usage**:
  ```bash
  radish status [--port <port_number>]
  ```
* **Examples**:
  ```bash
  radish status
  # Output: Server is RUNNING on port 6379 (received PONG).
  
  radish status --port 6385
  # Output: Could not connect to Radish server on port 6385. Is it running?
  ```

---

### 4. `radish stop`
Sends a graceful `SHUTDOWN` command to a running local database instance. The server will commit a final database memory snapshot to `dump.radish` before exiting safely.

* **Usage**:
  ```bash
  radish stop [--port <port_number>]
  ```
* **Examples**:
  ```bash
  radish stop
  # Output: Stop signal sent to server on port 6379.
  ```

---

### 5. `radish flush`
Sends a `FLUSHALL` signal to delete all keys currently loaded in database memory.

* **Usage**:
  ```bash
  radish flush [--port <port_number>]
  ```
* **Examples**:
  ```bash
  radish flush
  # Output: Database flushed on port 6379.
  ```

---

### 6. `radish inspect`
Queries a running server instance's diagnostics and telemetry using the `INFO` command, outputting metrics like uptime, process ID, client connections, OS architecture, and memory footprint.

* **Usage**:
  ```bash
  radish inspect [--port <port_number>]
  ```
* **Examples**:
  ```bash
  radish inspect
  ```
  * **Sample Output**:
    ```text
    # Server
    radish_version:0.1.0
    process_id:8492
    uptime_in_seconds:340
    os:windows
    arch:x86_64
    # Clients
    connected_clients:1
    # Memory
    used_memory:1820
    # Keyspace
    db0_keys:45
    ```

---

### 7. `radish service install`
Displays instructions and commands for registering the Radish server database engine as a persistent Windows Service.

* **Usage**:
  ```bash
  radish service install
  ```
* **Registration Steps**:
  1. Open a Command Prompt or PowerShell terminal as an **Administrator**.
  2. Run the printed `sc create` command to register the binary:
     ```powershell
     sc create "Radish" binPath= "C:\path\to\radish.exe start" start= auto
     ```
  3. Start the service:
     ```powershell
     sc start "Radish"
     ```

---

### 8. `radish studio`
Directly launches the graphical Tauri user interface (Radish Studio), equivalent to running the binary without arguments.

* **Usage**:
  ```bash
  radish studio
  ```
