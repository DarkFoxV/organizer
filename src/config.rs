use std::collections::HashSet;
use crate::utils::get_assets_path;
use log::{debug, error, info};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::error;
use std::fs;
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::sync::Mutex;
use crate::dtos::tag_dto::TagDTO;

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
    pub thumb_compression: Option<u8>,
    pub image_compression: Option<u8>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            language: "en".to_string(),
            items_per_page: 35,
            thumb_compression: Some(9),
            image_compression: Some(5),
        }
    }
}

/// In-memory UI state (search filters, pagination, scroll, etc.)
/// This is NOT persisted to disk - it's session-only
#[derive(Debug, Clone, Default)]
pub struct UIState {
    pub search_query: String,
    pub selected_tags: HashSet<TagDTO>,
    pub current_page: u64,
    pub scroll_offset: f32,
}

// ===================================
//         GLOBAL SINGLETONS
// ===================================

static SETTINGS: Lazy<RwLock<Settings>> = Lazy::new(|| {
    let settings = Settings::load();
    RwLock::new(settings)
});


static UI_STATE: Lazy<Mutex<UIState>> = Lazy::new(|| {
    Mutex::new(UIState::default())
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

// ===================================
//  UI STATE FUNCTIONS (IN-MEMORY ONLY)
// ===================================

/// Updates the search query
pub fn set_search_query(query: String) {
    UI_STATE.lock().unwrap().search_query = query;
}

/// Gets the current search query
pub fn get_search_query() -> String {
    UI_STATE.lock().unwrap().search_query.clone()
}

/// Updates the selected tags
pub fn set_selected_tags(tags: HashSet<TagDTO>) {
    UI_STATE.lock().unwrap().selected_tags = tags;
}

/// Gets the current selected tags
pub fn get_selected_tags() -> HashSet<TagDTO> {
    UI_STATE.lock().unwrap().selected_tags.clone()
}

/// Updates the current page
pub fn set_current_page(page: u64) {
    UI_STATE.lock().unwrap().current_page = page;
}

/// Gets the current page
pub fn get_current_page() -> u64 {
    UI_STATE.lock().unwrap().current_page
}

/// Updates the scroll offset
pub fn set_scroll_offset(offset: f32) {
    UI_STATE.lock().unwrap().scroll_offset = offset;
}

/// Gets the current scroll offset
pub fn get_scroll_offset() -> f32 {
    UI_STATE.lock().unwrap().scroll_offset
}

/// Resets the UI state to default (useful for "clear filters" functionality)
#[allow(dead_code)]
pub fn reset_ui_state() {
    *UI_STATE.lock().unwrap() = UIState::default();
}

/// Gets a complete clone of the UI state (useful for debugging)
#[allow(dead_code)]
pub fn get_ui_state() -> UIState {
    UI_STATE.lock().unwrap().clone()
}