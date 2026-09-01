use zbus::{proxy, Connection};

/// Talks to the scremetime GNOME Shell extension over D-Bus. The
/// extension exports this interface under GNOME Shell's own existing bus
/// name, org.gnome.Shell, rather than requesting a separate name of its
/// own, which is the normal pattern for shell extensions.
#[proxy(
    interface = "org.gnome.Shell.Extensions.Scremetime",
    default_service = "org.gnome.Shell",
    default_path = "/org/gnome/Shell/Extensions/Scremetime"
)]
pub trait AppFocus {
    /// The currently focused application's desktop file id (for example
    /// "firefox.desktop"), or an empty string if nothing is focused.
    fn get_focused_app(&self) -> zbus::Result<String>;

    #[zbus(signal)]
    fn focused_app_changed(&self, app_id: String);
}

pub async fn app_focus_proxy(session_conn: &Connection) -> zbus::Result<AppFocusProxy<'static>> {
    AppFocusProxy::new(session_conn).await
}
