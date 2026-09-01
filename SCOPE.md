# scremetime - scope tracking

## Goal
A fully featured, open source Linux system activity tracker for a GitHub/CV portfolio.
Tracks app level screen time with timestamps, battery consumption, and deep system
activity data. Three UI surfaces planned: macOS style native GUI, web dashboard,
system tray widget. Efficient, real time, local storage.

## Ground rules (do not drift from these)
- Explain as we build. Claude writes the code but narrates architecture decisions,
  tricky Rust concepts, and tradeoffs at each step, so Ahamed can describe and defend
  the project in a job interview even though Claude is doing most of the typing.
- README and any other documentation are written as if Ahamed wrote them personally:
  no emojis, no special characters, no em dashes, no sentence dashes.
- Code must be well structured, readable by an experienced developer, and secure.
- Ask before installing any dependency, package, or skill.
- Network data cap rule still applies: no non trivial downloads while on "SLT HOME"
  wifi outside 12am-7am, check first.
- Phased build. Do not start UI work until the data collection daemon and storage
  layer are complete and Ahamed understands how they work.
- If a side track comes up that is not the current step, name it explicitly before
  going deep on it.

## Locked in decisions
- Session environment: Wayland + GNOME (ubuntu:GNOME). Confirmed via
  XDG_SESSION_TYPE / loginctl on 2026-09-01.
- Consequence: Wayland does not allow a normal background process to query the
  focused window system wide (security by design). App level tracking requires a
  companion GNOME Shell extension (JavaScript) that runs inside the shell process
  and exposes the focused app over D-Bus to the daemon. This is a required
  architecture piece, not optional polish.
- Core daemon language: Rust. Chosen for performance and efficiency (the "most
  efficient, real time, long running service" requirement) and CV value. Accepted
  tradeoff: steepest learning curve of the options considered.
- Storage: SQLite, local only.
- Repo / directory name: scremetime (intentional stylized name, not a typo).
  Checked availability 2026-09-01: free on GitHub (org/user), npm, and crates.io.
- Location: ~/projects/scremetime

## Planned phases
1. Data collection daemon + SQLite schema (current phase)
   Confirmed Phase 1 data scope (2026-09-01):
   - App focus tracking via GNOME Shell extension + D-Bus. App name only, no
     window title (window titles can contain sensitive info like document
     names or browser tab contents, decided not worth the risk for Phase 1).
   - Battery stats via upower (charge %, charging state, power draw, time
     estimates)
   - CPU and memory usage over time
   - Idle time (seconds since last input) and lock/suspend/resume events
   - Disk I/O per process (read/write throughput), via /proc/<pid>/io, no
     special privileges needed
   - CLI to inspect collected data

   Deferred to a later phase: per-process network I/O. Linux has no simple
   per-process network counter (unlike disk I/O). The accurate way is eBPF,
   which needs root/CAP_BPF, a real barrier for anyone installing this from
   GitHub, and its own learning curve on top of Rust. The no-root way
   (socket-to-PID matching via /proc/net/tcp, like nethogs) is less precise.
   Decided 2026-09-01 to keep Phase 1 focused and revisit this later rather
   than block the core daemon on it.
2. macOS style native desktop GUI
   UI design decision (2026-09-01, for when this phase starts): default view
   should be simple and non technical (e.g. plain screen time and battery
   summaries, no raw numbers or jargon). A "nerd mode" toggle in settings
   reveals the detailed underlying data (exact timestamps, per sample values,
   etc.) for users who want it.
3. Web dashboard
4. System tray widget
5. Packaging, install docs, open source polish (LICENSE, CONTRIBUTING, screenshots)

## Security decision (2026-09-01)
Discussed encrypting the local database. Hashing ruled out (one way, would make
the data unreadable to the app itself). Full application-level encryption
deferred: doing it properly needs the key tied to the OS login keyring, not a
plaintext key file next to the encrypted database, which is a bigger feature
to build correctly later and worth documenting well when it happens. For now,
relying on: the user's full disk encryption, plus the daemon setting the data
directory to 0700 and the database file to 0600 (owner only).

## Roadmap
A higher level, public facing phase-by-phase roadmap now lives in
ROADMAP.md (created 2026-09-01). This SCOPE.md file stays the detailed
working log; ROADMAP.md is the polished version for GitHub visitors and
for tracking checklist-style progress at a glance.

## Status
- Phase: 1, data collection daemon.
- Done: SQLite schema (all 5 Phase 1 tables) created via daemon/src/db/schema.sql.
  Database connection setup with WAL mode and owner-only file permissions
  (daemon/src/db/mod.rs). Battery collector reading real hardware data from
  /sys/class/power_supply (daemon/src/collectors/battery.rs). System collector
  (CPU/mem) using the sysinfo crate, one long-lived System instance reused
  across polls since CPU usage is a delta between refreshes
  (daemon/src/collectors/system.rs). Both wired into main.rs as independent
  tokio intervals (battery every 5s, system every 3s while testing) running
  concurrently in one task via tokio::select!, no extra OS threads. Confirmed
  working by running the daemon and reading real rows back from both tables.
- Current step: decide next collector to build (idle/lock events, disk I/O,
  or the GNOME extension for app focus tracking).
- Not started: idle_events, disk_io_samples collectors, the GNOME Shell
  extension, CLI to query data, all UI phases (Phase 2-4), packaging/open
  source polish (Phase 5), keyring-backed encryption at rest.
