pub struct FocusAppInfo {
    pub exe_name: String,
    pub window_title: String,
}

/// Get the currently focused application info
pub fn current_focus_app() -> FocusAppInfo {
    FocusAppInfo {
        exe_name: "unknown.exe".into(),
        window_title: String::new(),
    }
}
