use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub input_method: InputMethodType,
    pub hotkeys: HotkeyConfig,
    pub auto_start: bool,
    pub show_status_bar: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InputMethodType {
    Telex,
    Vni,
    SimpleTelex,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyConfig {
    pub toggle_vietnamese: String,
    pub switch_input_method: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            input_method: InputMethodType::Telex,
            hotkeys: HotkeyConfig {
                toggle_vietnamese: "Ctrl+Shift".to_string(),
                switch_input_method: "Ctrl+Alt+V".to_string(),
            },
            auto_start: false,
            show_status_bar: true,
        }
    }
}

impl Config {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = Self::config_path()?;

        if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            let config: Config = toml::from_str(&content)?;
            Ok(config)
        } else {
            // Create default config if none exists
            let default_config = Config::default();
            default_config.save()?;
            Ok(default_config)
        }
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = Self::config_path()?;

        // Create config directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)?;
        fs::write(&config_path, content)?;
        Ok(())
    }

    fn config_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
        let home = std::env::var("HOME")?;
        Ok(PathBuf::from(home)
            .join(".config")
            .join("vaixkey")
            .join("config.toml"))
    }
}