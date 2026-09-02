# Roadmap

This document describes the planned direction of scremetime, phase by phase.
Detailed day to day working notes and decisions live in SCOPE.md. This file
is the higher level picture.

## Scope

scremetime tracks two things: app level screen time and battery. CPU,
memory, and disk I/O tracking were built during Phase 1 and then removed
by deliberate decision: this project is a screen time and battery
tracker, not a general system resource monitor. Idle time and screen
lock or suspend and resume detection stay in scope, since they support
accurate screen time accounting (knowing when you have stepped away)
rather than being general system stats.

## Phase 1: Data collection daemon

The background service that gathers everything. Written in Rust for
performance and efficiency, since it is meant to run continuously with
minimal resource use. Stores everything locally in a SQLite database.

- [x] SQLite schema for all Phase 1 data categories
- [x] Owner only file permissions on the database and its directory
- [x] Battery collector: charge percentage, charging state, power draw,
      time to empty or full, read directly from Linux battery hardware info
- [x] Idle time and lock or suspend or resume event collector
- [x] GNOME Shell extension for app focus tracking, since Wayland does not
      allow a normal background process to see which window is focused.
      Code complete, verification pending a session restart so GNOME Shell
      loads the new extension.
- [x] Command line tool to inspect collected data directly

Built and then removed (see Scope above): CPU and memory collector,
disk I/O collector, and their CLI and desktop app views.

Deferred: per process network I/O. The accurate method (eBPF) needs
elevated privileges, a real barrier for an open source tool other people
install. Also out of scope now given the battery/screen time focus above,
so unlikely to be revisited unless that focus changes.

## Phase 2: Desktop GUI

A native desktop application, styled after macOS Screen Time, for viewing
the collected data. Built with Tauri (Rust backend, React and TypeScript
frontend), chosen for full control over the visual design.

- [x] Choose a GUI toolkit: Tauri plus React and TypeScript
- [x] Daily, weekly, monthly, and all time view of app usage
- [x] Battery status view
- [x] Simple default view for a general audience
- [x] Nerd mode setting: toggle to reveal the underlying detailed data,
      exact timestamps, and raw sample values
- [x] First run onboarding screen: explains what the app does, and walks
      through enabling the GNOME extension if no app usage data exists yet
- [ ] App icons instead of plain text names
- [ ] Visual polish pass once the core views are confirmed working end
      to end with real data

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
- [x] LICENSE
- [ ] CONTRIBUTING guide
- [ ] README with screenshots and a clear description of what the project
      does and why it was built
- [ ] Packaging: a proper install script or distribution packages
- [ ] Keyring backed encryption at rest for the local database, as a
      documented security feature

## Ideas under consideration, not yet scheduled

- Export or backup of collected data
- Configurable polling intervals per collector
