# Roadmap

This document describes the planned direction of scremetime, phase by phase.
Detailed day to day working notes and decisions live in SCOPE.md. This file
is the higher level picture.

## Phase 1: Data collection daemon

The background service that gathers everything. Written in Rust for
performance and efficiency, since it is meant to run continuously with
minimal resource use. Stores everything locally in a SQLite database.

- [x] SQLite schema for all Phase 1 data categories
- [x] Owner only file permissions on the database and its directory
- [x] Battery collector: charge percentage, charging state, power draw,
      time to empty or full, read directly from Linux battery hardware info
- [x] CPU and memory collector
- [x] Disk I/O collector, per process read and write throughput
- [x] Idle time and lock or suspend or resume event collector
- [x] GNOME Shell extension for app focus tracking, since Wayland does not
      allow a normal background process to see which window is focused.
      Code complete, verification pending a session restart so GNOME Shell
      loads the new extension.
- [x] Command line tool to inspect collected data directly

Deferred out of Phase 1: per process network I/O. The accurate method
(eBPF) needs elevated privileges, a real barrier for an open source tool
other people install. Revisit this once the rest of the daemon is stable.

## Phase 2: Desktop GUI

A native desktop application, styled after macOS Screen Time, for viewing
the collected data.

- [ ] Choose a GUI toolkit
- [ ] Daily, weekly, monthly, and all time view of app usage
- [ ] Battery history and analytics view
- [ ] System resource history view
- [ ] Simple default view for a general audience
- [ ] Nerd mode setting: toggle to reveal the underlying detailed data,
      exact timestamps, and raw sample values

## Phase 3: Web dashboard

A browser based dashboard reading from the same local database, for
viewing analytics without opening the desktop app.

- [ ] Choose a web stack
- [ ] Local web server serving the dashboard, reading the same database
      as the daemon
- [ ] Same analytics views as the desktop GUI

## Phase 4: System tray widget

A lightweight tray icon for glanceable status without opening the full
app.

- [ ] Current battery status and active app at a glance
- [ ] Quick links into the desktop GUI or web dashboard

## Phase 5: Packaging and open source release

Getting the project ready for other people to install and use, and for
public release on GitHub.

- [ ] Installation instructions for common Linux distributions
- [ ] LICENSE
- [ ] CONTRIBUTING guide
- [ ] README with screenshots and a clear description of what the project
      does and why it was built
- [ ] Packaging: a proper install script or distribution packages
- [ ] Keyring backed encryption at rest for the local database, as a
      documented security feature

## Ideas under consideration, not yet scheduled

- Per process network I/O via eBPF, once the privilege and installation
  question has a good answer
- Export or backup of collected data
- Configurable polling intervals per collector
