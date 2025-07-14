use std::error;
use std::fs;
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use log::{debug, error, info};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use crate::utils::get_assets_path;

/// Main structure holding runtime settings
#[derive(Debug, Clone)]
pub struct Settings {
    pub config: Config,
}

impl Settings {
    /// Loads settings from disk or uses defaults if the file is missing or invalid
    pub fn load() -> Self {
        let config = Self::load_config();
        Settings { config }
    }

    /// Reads config.json and deserializes into Config
    fn load_config() -> Config {
        let config_path = get_assets_path().join("config.json");

        fs::read_to_string(&config_path)
            .and_then(|content| serde_json::from_str(&content).map_err(Into::into))
            .unwrap_or_else(|err| {
                error!("Failed to load config.json: {}. Using default config.", err);
                Config::default()
            })
    }

    /// Saves the current settings to config.json
    pub fn save(&self) -> Result<(), Box<dyn error::Error>> {
        let config_path = get_assets_path().join("config.json");
        let config_json = serde_json::to_string_pretty(&self.config)?;

        debug!("Saving config to {}", config_path.display());
        debug!("Config JSON: {}", config_json);

        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(&config_path, config_json)?;
        info!("Config saved");
        Ok(())
    }
}

/// Serializable structure for app config
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub theme: String,
    pub language: String,
    pub items_per_page: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            language: "en".to_string(),
            items_per_page: 35,
        }
    }
}

// ===================================
//         GLOBAL CONFIG SINGLETON
// ===================================

static SETTINGS: Lazy<RwLock<Settings>> = Lazy::new(|| {
    let settings = Settings::load();
    RwLock::new(settings)
});

/// Gets a read-only lock on the global Settings
pub fn get_settings() -> RwLockReadGuard<'static, Settings> {
    SETTINGS
        .read()
        .expect("Failed to acquire read lock on SETTINGS")
}

/// Gets a writable lock on the global Settings
pub fn get_settings_mut() -> RwLockWriteGuard<'static, Settings> {
    SETTINGS
        .write()
        .expect("Failed to acquire write lock on SETTINGS")
}
