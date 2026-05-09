use std::path::PathBuf;

pub struct AppPaths {
    pub config: PathBuf,
    pub license: PathBuf,
    pub history_db: PathBuf,
    pub logs_dir: PathBuf,
    pub prompts_dir: PathBuf,
}

impl Default for AppPaths {
    fn default() -> Self {
        Self::new()
    }
}

impl AppPaths {
    pub fn new() -> Self {
        let base = dirs::data_dir()
            .or_else(dirs::config_dir)
            .unwrap_or_else(|| PathBuf::from("."))
            .join("WhisperKey");

        let logs_dir = base.join("logs");
        let prompts_dir = base.join("prompts");

        // Auto-create directories on first access
        std::fs::create_dir_all(&logs_dir).ok();
        std::fs::create_dir_all(&prompts_dir).ok();

        Self {
            config: base.join("config.json"),
            license: base.join("license.dat"),
            history_db: base.join("history.db"),
            logs_dir,
            prompts_dir,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paths_start_with_appdata() {
        let paths = AppPaths::new();
        let appdata = dirs::data_dir()
            .or_else(dirs::config_dir)
            .unwrap_or_else(|| PathBuf::from("."));

        let expected_base = appdata.join("WhisperKey");

        assert!(paths.config.starts_with(&expected_base));
        assert!(paths.license.starts_with(&expected_base));
        assert!(paths.history_db.starts_with(&expected_base));
        assert!(paths.logs_dir.starts_with(&expected_base));
        assert!(paths.prompts_dir.starts_with(&expected_base));
    }

    #[test]
    fn test_dirs_created() {
        let paths = AppPaths::new();
        assert!(paths.logs_dir.exists());
        assert!(paths.prompts_dir.exists());
    }
}
