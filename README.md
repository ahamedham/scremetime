# scremetime

A system activity tracker for Linux. The goal is something like macOS
Screen Time, but for Linux: how much time you spend in each app and your
battery history, stored locally on your own machine.

This project is under active development and is not yet ready for
everyday use. See ROADMAP.md for what is planned and what is already
working.

## Why this exists

Linux does not have a good built in equivalent to the screen time and
battery history tools that exist on macOS and iOS. This project is an
attempt to build one properly: efficient, focused, and fully local, with
no data leaving your machine unless you choose to export it yourself.

It is also a personal project I am building and documenting openly as
part of my own portfolio, with the intent of eventually being usable by
anyone who wants it.

## Current status

The project is being built in phases. Phase 1, the background data
collection daemon, is essentially complete. Phase 2, a native desktop
app, is in progress.

Working so far:
- A Rust daemon that collects battery statistics (charge percentage,
  charging state, power draw, time to empty or full) directly from Linux
  battery hardware information
- App level focus tracking (which app you are using and for how long),
  via a companion GNOME Shell extension
- Idle time and screen lock or suspend and resume detection, so screen
  time accounting can account for when you have stepped away
- All collected data is written to a local SQLite database with
  restrictive file permissions so only your own user account can read it
- A desktop app (Tauri plus React) showing daily, weekly, monthly, and
  all time app usage and battery status, with a "Nerd Mode" toggle to
  see the underlying raw data

Deliberately out of scope: CPU, memory, and disk I/O tracking were built
and then removed. This project is about screen time and battery, not a
general system resource monitor. Not yet built: the web dashboard and
the system tray widget. Full detail is in ROADMAP.md.

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

## Enabling app focus tracking

App level tracking needs the companion GNOME Shell extension in
gnome-extension/. Install it by linking or copying that folder into
GNOME's extensions directory under its uuid, then enable it:

```
ln -s "$(pwd)/gnome-extension" ~/.local/share/gnome-shell/extensions/scremetime@ahamedham.github.io
```

GNOME Shell only scans for newly added extensions at login, so after
this you need to log out and back in (there is no in place shell restart
on Wayland), then run:

```
gnome-extensions enable scremetime@ahamedham.github.io
```

Without this step, the daemon still runs and still collects everything
else. It logs a message saying app focus tracking is unavailable and
picks it up automatically later if you enable the extension while the
daemon keeps running.

## Inspecting collected data

A small CLI reads from the same database:

```
cargo run --bin scremetime -- apps --period today
cargo run --bin scremetime -- battery --limit 10
cargo run --bin scremetime -- idle
```

## Desktop app

A native desktop app (Tauri, with a React and TypeScript frontend) reads
from the same database:

```
cd desktop
npm install
npm run tauri dev
```

On Debian or Ubuntu based distributions, Tauri needs a few system
packages first:

```
sudo apt-get install -y libwebkit2gtk-4.1-dev libjavascriptcoregtk-4.1-dev libsoup-3.0-dev libgtk-3-dev build-essential curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev
```

## Data and privacy

Everything scremetime collects is stored locally on your machine, in a
SQLite database that only your own user account can read. Nothing is sent
anywhere. Window titles are intentionally not collected, only application
names, since window titles can contain sensitive information such as
document names or browser tab contents.

## License

Licensed under the Apache License, Version 2.0. See LICENSE for the full
text.
