use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{env, fs, path::PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppRule {
    pub exe: String,
    pub layer: u8,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub vendor_id: u16,
    pub product_id: u16,
    pub default_layer: u8,
    pub rules: Vec<AppRule>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            vendor_id: 0,
            product_id: 0,
            default_layer: 0,
            rules: Vec::new(),
        }
    }
}

fn config_path() -> PathBuf {
    let appdata = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
    let app_path = PathBuf::from(appdata).join("dynamic-layer-switcher");
    std::fs::create_dir_all(&app_path).ok();
    let config_path = app_path.join("config.cfg");
    config_path
}

fn read_config(path: &PathBuf) -> Result<Config, String> {
    let config_data = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_json::from_str(&config_data).map_err(|e| e.to_string())
}

fn write_config(config: &Config) -> Result<(), String> {
    let write_string = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;
    std::fs::write(config_path(), &write_string).map_err(|e| e.to_string())
}

pub fn get_config() -> Config {
    let path = config_path();
    let config = read_config(&path);
    if config.is_err() {
        write_config(&Config::default()).ok();
    }
    config.unwrap_or_else(|_| Config::default())
}
