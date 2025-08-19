# t_trace 🚀 [![Rust](https://github.com/M-Komorek/t_trace/actions/workflows/rust.yml/badge.svg?branch=main)](https://github.com/M-Komorek/t_trace/actions/workflows/rust.yml)

`t_trace` is a high-performance, command-line statistics tracker for your shell.

It silently observes your command-line usage and provides fast, insightful statistics on which commands you run, how often you run them, and how long they take, all with a negligible performance impact on your interactive shell.

## Demo
`t_trace` collects statistics in the background and lets you view them instantly with the stats command.
``` bash
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

## Features
- ⚡️ Blazing Fast: Written in **Rust** with a **client-daemon architecture** over a **Unix Domain Socket**. The **shell hooks** are incredibly lightweight and exit in microseconds, ensuring zero perceived lag in your terminal.
- 📊 Insightful Stats: Tracks execution count, total time, mean time, last run time, and success/failure rates for every command.
- ✨ Seamless Integration: A simple, one-line eval command is all you need to hook `t_trace` into your shell. The daemon starts automatically and manages itself.
- 💾 Robust & Persistent: The daemon process runs reliably in the background. It saves state gracefully on shutdown, so your statistics survive reboots.

## Installation
TODO

## How It Works
`t_trace` uses a performant client-daemon architecture to avoid slowing down your shell.

- The Daemon: A single background process acts as the "brain". It listens on a Unix Domain Socket, manages all in-memory state, aggregates statistics, and handles saving data to disk.
- The Client: An extremely lightweight binary invoked by your shell's hooks (trap `DEBUG` and `PROMPT_COMMAND`). Its only job is to send a tiny message to the daemon and exit immediately.
- Communication: All communication happens over a high-speed, low-latency Unix Domain Socket, which avoids network overhead and is perfect for local inter-process communication.

## License
This project is dual-licensed under the terms of both the MIT license and the Apache License 2.0. You may choose to use it under either license.
