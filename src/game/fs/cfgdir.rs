use crate::game::fs::smartdir::SmartDir;
use std::path::PathBuf;

/// Returns the configuration app directory, e.g. APPDATA on windows
pub fn acquire_config_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        std::env::var_os("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|| panic!("APPDATA is not set"))
    }

    #[cfg(target_os = "linux")]
    {
        let dir = if let Some(xdg) = std::env::var_os("XDG_CONFIG_HOME") {
            PathBuf::from(xdg)
        } else if let Some(home) = std::env::var_os("HOME") {
            PathBuf::from(home).join(".config")
        } else {
            panic!("Neither XDG_CONFIG_HOME nor HOME is set")
        };

        //my debian machine doesnt automatically have .config so just create it lol
        if !dir.exists() {
            std::fs::create_dir_all(&dir)
                .unwrap_or_else(|e| panic!("Failed to create config dir {:?}: {}", dir, e));
        }

        dir
    }

    #[cfg(target_os = "macos")]
    {
        if let Some(home) = std::env::var_os("HOME") {
            PathBuf::from(home)
                .join("Library")
                .join("Application Support")
        } else {
            panic!("HOME is not set on macOS")
        }
    }
}

pub fn acquire_config_smart_dir() -> SmartDir {
    SmartDir::new(acquire_config_dir())
}
