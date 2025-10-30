# t_trace 🚀 [![Rust](https://github.com/M-Komorek/t_trace/actions/workflows/rust.yml/badge.svg?branch=main)](https://github.com/M-Komorek/t_trace/actions/workflows/rust.yml)

The `t_trace` is a high-performance, command-line statistics tracker for your shell.

It silently observes your command-line usage and provides fast, insightful statistics on which commands you run, how often you run them, and how long they take, all with a negligible performance impact on your interactive shell.

## Demo
The `t_trace` collects statistics in the background and lets you view them instantly with the stats command.
``` bash
$ t_trace stats
┌─────────────┬───────────────┬────────────┬─────────────┬───────────┬───────────┐
│ Command     ┆ Success Count ┆ Fail Count ┆ Total Time  ┆ Mean Time ┆ Last Time │
╞═════════════╪═══════════════╪════════════╪═════════════╪═══════════╪═══════════╡
│ cargo build ┆ 12            ┆ 3          ┆ 13m 57.300s ┆ 55.820s   ┆ 1m 2.100s │
├╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌┤
│ git log     ┆ 3             ┆ 0          ┆ 16.575s     ┆ 5.525s    ┆ 3.116s    │
├╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌┤
│ git status  ┆ 58            ┆ 0          ┆ 4.937s      ┆ 85.123ms  ┆ 75.450ms  │
├╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌┤
│ ls -l       ┆ 112           ┆ 0          ┆ 504.112ms   ┆ 4.501ms   ┆ 3.987ms   │
├╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌┤
│ cd src/     ┆ 1             ┆ 0          ┆ 3.414ms     ┆ 3.414ms   ┆ 3.414ms   │
└─────────────┴───────────────┴────────────┴─────────────┴───────────┴───────────┘
```
Quickly find all your `git` related commands:
```bash
$ t_trace stats -g git
┌─────────────┬───────────────┬────────────┬─────────────┬───────────┬───────────┐
│ Command     ┆ Success Count ┆ Fail Count ┆ Total Time  ┆ Mean Time ┆ Last Time │
╞═════════════╪═══════════════╪════════════╪═════════════╪═══════════╪═══════════╡
│ cargo build ┆ 12            ┆ 3          ┆ 13m 57.300s ┆ 55.820s   ┆ 1m 2.100s │
├╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌┤
│ git log     ┆ 3             ┆ 0          ┆ 16.575s     ┆ 5.525s    ┆ 3.116s    │
├╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌┤
│ git status  ┆ 58            ┆ 0          ┆ 4.937s      ┆ 85.123ms  ┆ 75.450ms  │
└─────────────┴───────────────┴────────────┴─────────────┴───────────┴───────────┘
```

## Features
- ⚡️ Blazing Fast: Written in **Rust** with a **client-daemon architecture** over a **Unix Domain Socket**. The **shell hooks** are incredibly lightweight and exit in microseconds, ensuring zero perceived lag in your terminal.
- 📊 Insightful Stats: Tracks execution count, total time, mean time, last run time, and success/failure rates for every command.
- ✨ Seamless Integration: A simple, one-line eval command is all you need to hook the `t_trace` into your shell. The daemon starts automatically and manages itself.
- 💾 Robust & Persistent: The daemon process runs reliably in the background. It saves state gracefully on shutdown, so your statistics survive reboots.

## How to configure
You need the [Rust toolchain](https://www.rust-lang.org/tools/install) installed on your system.
If the toolchain is ready, you need to complete two steps: installation and one-time setup.

### 1. Installation
#### a) From Crates.io (Recommended)
``` Bash
cargo install t_trace
```
#### b) From Source (Alternative)
``` Bash
git clone https://github.com/M-Komorek/t_trace.git
cd t_trace
cargo install --path .
```

### 2. One-Time Setup
To hook the `t_trace` into your shell, add the following line to the end of your `~/.bashrc`:
``` Bash
eval "$(t_trace init bash)"
```

Then, either restart your shell or run `source ~/.bashrc` to apply the changes.

That's it! The `t_trace` will now start automatically with your shell session and begin tracking commands.

## Usage

Once installed and set up, the `t_trace` works silently in the background. You can interact with it using these commands:

| Command | Description |
| :--- | :--- |
| `t_trace stats` | Display your aggregated command statistics in a formatted table. |
| `t_trace stats -g <phrase>` | Filter the stats to show only commands containing `<phrase>`. |
| `t_trace daemon health-check` | Check if the `t_trace` background daemon is running and responsive. |
| `t_trace daemon run`| Manually start the daemon (usually handled automatically by the shell script). |
| `t_trace daemon stop` | Stop the daemon gracefully, ensuring all collected data is saved to disk. |

## Under the hood
The `t_trace` uses a performant client-daemon architecture to avoid slowing down your shell.

- **Shell Hook Integration:** The tool hooks into Bash's execution cycle using the standard trap DEBUG and PROMPT_COMMAND mechanisms. This allows it to reliably capture a command just before it runs and its exit code just after it finishes, forming the basis of its time tracking.
- **High-Performance Client-Daemon Architecture:** To ensure zero shell latency, all heavy lifting (state management, calculations, file I/O) is handled by a single, long-running daemon process. The shell hooks only execute an extremely fast, compiled Rust client whose only job is to send a message and exit immediately.
- **Optimized Communication:** The client and daemon communicate via a Unix Domain Socket (UDS). This is a high-speed, low-latency Inter-Process Communication (IPC) method that operates entirely within the OS kernel, bypassing the network stack for maximum efficiency on a local machine.
- **Concurrent and Asynchronous Daemon:** The daemon is built with Tokio, Rust's modern async runtime. It uses an event loop with tokio::select! to concurrently listen for new client connections and system shutdown signals (SIGINT/SIGTERM). This non-blocking model allows a single thread to handle hundreds of connections efficiently and ensures robust, graceful termination.
- **In-Memory State with Thread Safety:** All command statistics are held in memory within the daemon for fast access. The daemon state is wrapped in an Arc<Mutex<...>> to guarantee safe, concurrent access from multiple connection-handling tasks without race conditions.
- **Robust JSON Persistence:** The daemon's in-memory state is serialized to a human-readable stats.json file using the powerful Serde library. The system uses an "atomic save" pattern (write to a temporary file, then rename) to prevent data corruption if the process is terminated unexpectedly during a write.
- **Client-Side Presentation Logic:** When you run `t_trace stats`, the client fetches the entire dataset from the daemon. All filtering (for the --grep flag), sorting, and table formatting (using comfy-table) happen on the client side. This keeps the daemon's responsibility simple and focused: be a fast, dumb data store.
