# scremetime

A system activity tracker for Linux. The goal is something like macOS
Screen Time, but for Linux, and going further: app usage over time,
battery consumption, and general system activity, all stored locally on
your own machine.

This project is under active development and is not yet ready for
everyday use. See ROADMAP.md for what is planned and what is already
working.

## Why this exists

Linux does not have a good built in equivalent to the screen time and
battery history tools that exist on macOS and iOS. This project is an
attempt to build one properly: efficient, detailed, and fully local, with
no data leaving your machine unless you choose to export it yourself.

It is also a personal project I am building and documenting openly as
part of my own portfolio, with the intent of eventually being usable by
anyone who wants it.

## Current status

The project is being built in phases. Phase 1, the background data
collection daemon, is in progress.

Working so far:
- A Rust daemon that collects battery statistics (charge percentage,
  charging state, power draw, time to empty or full) directly from Linux
  battery hardware information
- CPU and memory usage collection
- All collected data is written to a local SQLite database with
  restrictive file permissions so only your own user account can read it

Not yet built: app level focus tracking, idle time and lock or suspend
detection, disk I/O tracking, the desktop interface, the web dashboard,
and the system tray widget. Full detail is in ROADMAP.md.

## Why Rust

The daemon is written in Rust because it is meant to run continuously in
the background with minimal CPU and memory overhead. Rust gives that
level of efficiency without a garbage collector, along with strong
guarantees about memory safety, which matters for a long running system
service.

## A note on Wayland

This project is being developed and tested on Wayland with GNOME. Wayland
intentionally does not allow a normal background process to query which
window currently has focus, for security reasons. Because of this, app
level tracking requires a small companion GNOME Shell extension that runs
inside the shell itself and reports the focused application over D-Bus.
This is a real architectural requirement of the project, not an
afterthought, and is tracked in ROADMAP.md.

## Building the daemon

Requires a recent Rust toolchain (install via rustup).

```
cd daemon
cargo build
cargo run
```

The daemon will create its database at
`~/.local/share/scremetime/data.db` on first run.

## Data and privacy

Everything scremetime collects is stored locally on your machine, in a
SQLite database that only your own user account can read. Nothing is sent
anywhere. Window titles are intentionally not collected, only application
names, since window titles can contain sensitive information such as
document names or browser tab contents.

## License

Licensed under the Apache License, Version 2.0. See LICENSE for the full
text.
