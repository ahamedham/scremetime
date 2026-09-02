# scremetime - scope tracking

## Goal
An open source Linux screen time and battery tracker for a GitHub/CV portfolio.
Tracks app level screen time with timestamps and battery consumption. Three UI
surfaces planned: macOS style native GUI, web dashboard, system tray widget.
Efficient, real time, local storage.

Scope narrowed 2026-09-02: CPU, memory, and disk I/O tracking were built during
Phase 1 (system.rs and disk_io.rs collectors, their database tables, CLI
subcommands, and desktop app panels) and then removed by explicit user
decision: "i dont want records of the cpu, memory usage and all. only battery,
screen time." Idle time and lock/suspend/resume detection were kept, since
they support accurate screen time accounting (detecting when the user has
stepped away) rather than being general system resource stats, which is what
the user was actually objecting to. See ROADMAP.md's Scope section for the
user facing version of this decision. All code and already collected data for
the removed collectors was deleted, not just disabled, per the user's explicit
choice when asked.

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
- Phase: 2, desktop GUI (Tauri + React + TypeScript). Phase 1 code complete.
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
  Disk I/O collector added (daemon/src/collectors/disk_io.rs), polling
  /proc/<pid>/io for every process and storing deltas since the last poll
  rather than the kernel's cumulative counters, and only when nonzero, so
  idle processes do not fill the database. Verified with a real 20MB dd
  write showing up as an exact byte-accurate row attributed to the correct
  process.
- GitHub repo created and pushed: https://github.com/ahamedham/scremetime,
  public, Apache 2.0 license (chosen 2026-09-01 since this repo is meant to
  be genuinely open source, unlike the "portfolio only" convention used for
  other repos in [[project-portfolio-cv-launch]]). README and ROADMAP written
  to be public facing.
  Idle/lock/suspend collector added (daemon/src/collectors/idle.rs), first
  use of D-Bus in the project via the zbus crate. Idle time polls GNOME
  Mutter's IdleMonitor (GNOME specific, accepted since app tracking already
  needs GNOME Shell). Lock/unlock listens for org.gnome.ScreenSaver's
  ActiveChanged signal. Suspend/resume listens for systemd-logind's
  PrepareForSleep signal on the system bus (deliberately not GNOME
  specific, since logind is present on effectively all modern Linux
  distributions). Verified for real: the daemon correctly detected genuine
  idle_start after the user had been away from keyboard and mouse for over
  60 seconds, no artificial test needed.
  GNOME Shell extension written (gnome-extension/extension.js, GNOME 45+
  ES module style for GNOME Shell 46, confirmed the local shell version).
  Watches Shell.WindowTracker's focus-app property (gives a stable app id
  directly, no window title parsing needed) and exports it over D-Bus
  under GNOME Shell's own existing bus name, at
  /org/gnome/Shell/Extensions/Scremetime. Rust side added
  (daemon/src/collectors/app_focus.rs) plus app_sessions start/end
  functions in db/mod.rs and wiring in main.rs, including graceful
  degradation: if the extension is not enabled, the daemon logs a clear
  message and keeps running every other collector rather than crashing,
  and the D-Bus signal subscription stays valid so app tracking picks up
  automatically later without a daemon restart if the extension gets
  enabled afterward.
- Blocked on a session restart: GNOME Shell only scans for brand new
  extension UUIDs at login, not while the session is running, and Wayland
  has no in-place shell restart like X11 did. Confirmed this by installing
  the extension (symlinked into
  ~/.local/share/gnome-shell/extensions/scremetime@ahamedham.github.io)
  and finding gnome-extensions and the shell's own GetExtensionInfo D-Bus
  call both report it as not found while the session stays live. Verified
  the daemon's graceful degradation path handles this correctly: it prints
  a clear "not enabled yet" message and every other collector keeps
  working. Did not log the user out to force a test, since that would
  close their active session and open windows without asking first.
  CLI added (daemon/src/bin/cli.rs, binary name "scremetime"). Restructured
  the crate into a library (daemon/src/lib.rs exposing collectors and db)
  plus two binaries (scremetime-daemon and scremetime) so both can share
  the same database code, standard Rust pattern for this shape of project.
  Read side query functions added to db/mod.rs: app usage totals, recent
  battery/system/disk/idle rows. Verified against the real data collected
  so far: apps, battery, system, disk, and idle subcommands all print
  correctly formatted real rows (apps correctly reports none yet, since
  that is still pending the extension).
- Phase 1 is now code complete. Only remaining step: user needs to log out
  and back in (or reboot) at a time that suits them, then run
  `gnome-extensions enable scremetime@ahamedham.github.io` and restart the
  daemon to verify real app focus tracking end to end.
- Not started: all UI phases (Phase 2-4), packaging/open source polish
  (Phase 5), keyring-backed encryption at rest.
- Ground rule reaffirmed 2026-09-01: user asked to "do everything" per the
  roadmap in one go; pushed back and agreed instead to keep building at a
  steady pace without re-confirming small steps, but still pause at real
  architecture forks (GUI toolkit choice, web stack choice) and is already
  proceeding through Phase 1 collectors and the initial GitHub push under
  that agreement.

- 2026-09-02: GUI toolkit fork resolved. User chose Tauri (Rust backend
  plus web frontend) over GTK4/libadwaita and egui/iced, specifically for
  full pixel level control over the macOS Screen Time look, accepting the
  tradeoff of less "genuinely native" than GTK. Defaulted to React plus
  TypeScript for the frontend without a separate question, since that is
  an implementation detail within the chosen approach rather than a fork
  with real tradeoffs.
- Tauri needed sizeable system packages (webkit2gtk-4.1-dev and related)
  plus npm/cargo downloads. This coincided with being on "SLT HOME" wifi
  at 11:57pm, 3 minutes before the unmetered midnight-7am window per
  [[feedback-network-data-caps]]. Flagged size and timing to the user per
  that rule rather than proceeding; user chose to wait the few minutes for
  midnight. Installed the apt packages by handing the user the exact sudo
  command to run themselves in their own terminal, since Claude cannot and
  should not enter their sudo password.
- Scaffolded desktop/ as a Tauri + React + TypeScript app (npm create
  tauri-app). Rather than duplicating database query logic, desktop/src-tauri
  depends on the daemon crate via a local path dependency and reuses the
  exact same db module functions the CLI uses (one source of truth for
  reads). Extracted a shared time_util module (today/week/month start
  timestamps, a Period enum) into the daemon lib, used by both the CLI's
  --period flag and the desktop app's period selector, rather than
  duplicating that date math a third time.
- Built the initial UI: macOS style design (rounded cards, soft shadows,
  system font stack, light/dark via prefers-color-scheme), a period
  segmented control (Today/Week/Month/All Time), a Screen Time style app
  usage list with proportional bars, a quick stats row, and a Nerd Mode
  toggle revealing raw data tables, per the user's earlier explicit request
  for a non-technical default view plus a detailed toggle. Verified the
  frontend structurally via a browser preview of the Vite dev server
  (Tauri's invoke() naturally fails outside the real app, confirmed via
  console/accessibility tree that this fails gracefully into the existing
  error banner rather than crashing) and then, since this session has a
  real display (WAYLAND_DISPLAY/DISPLAY confirmed set), actually launched
  the real native Tauri window on the user's screen with `npm run tauri
  dev` to verify the genuine article rather than only the web approximation.

- 2026-09-02, user request: "i dont want records of the cpu, memory usage
  and all. only battery, screen time." See the updated Goal section above
  for the full reasoning captured at the time. Asked one clarifying round
  (keep idle/lock/suspend since it supports screen time accuracy: yes;
  remove code and data, not just disable: yes) before acting, per
  [[feedback-teacher-mode]]'s emphasis on evaluating requests rather than
  silently complying, but this was a legitimate straightforward scope cut
  so it did not warrant pushback, only clarification of edges.
  Removed: daemon/src/collectors/system.rs and disk_io.rs entirely, their
  Cargo.toml dependency (sysinfo), their db.rs structs/functions/tables
  (added explicit DROP TABLE IF EXISTS statements to schema.sql so the
  already-collected data is actually deleted from existing local databases
  on next daemon start, not just stopped from growing further), their
  main.rs collectors/intervals/select! branches, their CLI subcommands,
  their Tauri commands, and their frontend types/API calls/UI (QuickStats
  component deleted and replaced with a simpler BatteryCard, since a
  3-column stats grid did not make sense with only one stat left in it;
  NerdPanel's System and Disk I/O table sections removed).
  Verified: daemon and desktop/src-tauri both build clean after the
  removal, frontend type-checks clean, and the live running `tauri dev`
  process (still running from the earlier launch) picked up the changes
  via its file watcher and rebuilt successfully, confirmed by reading its
  log and the running process id.
