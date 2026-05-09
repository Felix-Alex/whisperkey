/// System tray module
/// Creates tray icon with context menu (mode switching, settings, pause, quit)
pub struct TrayManager {
    _priv: (),
}

impl TrayManager {
    pub fn new() -> Self {
        Self { _priv: () }
    }
}

impl Default for TrayManager {
    fn default() -> Self {
        Self::new()
    }
}
