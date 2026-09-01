use zbus::{proxy, Connection};

/// How long with no keyboard or mouse input before we consider the user
/// idle. GNOME's own screen dimming defaults are in a similar range; this
/// is a reasonable starting point and can become configurable later.
const IDLE_THRESHOLD_MS: u64 = 60_000;

/// Mutter (GNOME's compositor) exposes idle time directly over D-Bus, on
/// the session bus. This is GNOME specific, which is an accepted
/// constraint of this project since app focus tracking already requires
/// a GNOME Shell extension.
#[proxy(
    interface = "org.gnome.Mutter.IdleMonitor",
    default_service = "org.gnome.Mutter.IdleMonitor",
    default_path = "/org/gnome/Mutter/IdleMonitor/Core"
)]
pub trait IdleMonitor {
    /// Milliseconds since the last keyboard or mouse input.
    fn get_idletime(&self) -> zbus::Result<u64>;
}

/// The screen lock state, also on the session bus. GNOME implements
/// org.gnome.ScreenSaver; ActiveChanged(true) means the screen just
/// locked, ActiveChanged(false) means it just unlocked.
#[proxy(
    interface = "org.gnome.ScreenSaver",
    default_service = "org.gnome.ScreenSaver",
    default_path = "/org/gnome/ScreenSaver"
)]
pub trait ScreenSaver {
    #[zbus(signal)]
    fn active_changed(&self, new_value: bool);
}

/// Suspend and resume, from systemd-logind on the system bus. This is
/// deliberately not GNOME specific: logind is present on essentially
/// every modern Linux distribution regardless of desktop environment,
/// which matters for an open source tool other people will install.
#[proxy(
    interface = "org.freedesktop.login1.Manager",
    default_service = "org.freedesktop.login1",
    default_path = "/org/freedesktop/login1"
)]
pub trait LoginManager {
    /// Fired twice per suspend cycle: once with start = true right before
    /// the machine sleeps, once with start = false right after it wakes.
    #[zbus(signal)]
    fn prepare_for_sleep(&self, start: bool);
}

pub enum IdleEvent {
    IdleStart,
    IdleEnd,
    Locked,
    Unlocked,
    Suspended,
    Resumed,
}

impl IdleEvent {
    pub fn as_db_str(&self) -> &'static str {
        match self {
            IdleEvent::IdleStart => "idle_start",
            IdleEvent::IdleEnd => "idle_end",
            IdleEvent::Locked => "lock",
            IdleEvent::Unlocked => "unlock",
            IdleEvent::Suspended => "suspend",
            IdleEvent::Resumed => "resume",
        }
    }
}

pub struct IdleWatcher {
    idle_monitor: IdleMonitorProxy<'static>,
    was_idle: bool,
}

impl IdleWatcher {
    pub async fn new(session_conn: &Connection) -> zbus::Result<Self> {
        let idle_monitor = IdleMonitorProxy::new(session_conn).await?;
        Ok(Self {
            idle_monitor,
            was_idle: false,
        })
    }

    /// Polls current idle time and reports an event only when the idle
    /// state has actually crossed the threshold since the last poll, not
    /// on every call.
    pub async fn poll(&mut self) -> Option<IdleEvent> {
        let idle_ms = self.idle_monitor.get_idletime().await.ok()?;
        let is_idle_now = idle_ms >= IDLE_THRESHOLD_MS;

        if is_idle_now && !self.was_idle {
            self.was_idle = true;
            Some(IdleEvent::IdleStart)
        } else if !is_idle_now && self.was_idle {
            self.was_idle = false;
            Some(IdleEvent::IdleEnd)
        } else {
            None
        }
    }
}

pub async fn screen_saver_proxy(session_conn: &Connection) -> zbus::Result<ScreenSaverProxy<'static>> {
    ScreenSaverProxy::new(session_conn).await
}

pub async fn login_manager_proxy(system_conn: &Connection) -> zbus::Result<LoginManagerProxy<'static>> {
    LoginManagerProxy::new(system_conn).await
}
