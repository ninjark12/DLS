use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

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

impl Config {
    /// Resolve which layer a given focused executable should map to.
    /// Falls back to `default_layer` when no rule matches. Matching is
    /// case-insensitive because Windows exe names vary in capitalization.
    pub fn layer_for(&self, exe: &str) -> u8 {
        self.rules
            .iter()
            .find(|r| r.exe.eq_ignore_ascii_case(exe))
            .map(|r| r.layer)
            .unwrap_or(self.default_layer)
    }
}

fn config_path() -> PathBuf {
    let appdata = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
    let app_path = PathBuf::from(appdata).join("dynamic-layer-switcher");
    std::fs::create_dir_all(&app_path).ok();
    app_path.join("config.cfg")
}

fn read_config(path: &Path) -> Result<Config, String> {
    let config_data = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_json::from_str(&config_data).map_err(|e| e.to_string())
}

fn write_config_to(path: &Path, config: &Config) -> Result<(), String> {
    let write_string = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    std::fs::write(path, &write_string).map_err(|e| e.to_string())
}

pub fn write_config(config: &Config) -> Result<(), String> {
    write_config_to(&config_path(), config)
}

pub fn get_config() -> Config {
    let path = config_path();
    match read_config(&path) {
        Ok(config) => config,
        Err(_) => {
            write_config(&Config::default()).ok();
            Config::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule(exe: &str, layer: u8) -> AppRule {
        AppRule {
            exe: exe.to_string(),
            layer,
            label: exe.to_string(),
        }
    }

    fn sample_config() -> Config {
        Config {
            vendor_id: 0xCB10,
            product_id: 0x1756,
            default_layer: 0,
            rules: vec![rule("clipstudiopaint.exe", 3), rule("code.exe", 1)],
        }
    }

    #[test]
    fn matched_rule_returns_its_layer() {
        let cfg = sample_config();
        assert_eq!(cfg.layer_for("clipstudiopaint.exe"), 3);
        assert_eq!(cfg.layer_for("code.exe"), 1);
    }

    #[test]
    fn matching_is_case_insensitive() {
        let cfg = sample_config();
        assert_eq!(cfg.layer_for("ClipStudioPaint.exe"), 3);
        assert_eq!(cfg.layer_for("CODE.EXE"), 1);
    }

    #[test]
    fn unmatched_exe_falls_back_to_default_layer() {
        let mut cfg = sample_config();
        cfg.default_layer = 5;
        assert_eq!(cfg.layer_for("notepad.exe"), 5);
    }

    #[test]
    fn empty_config_always_returns_default_layer() {
        let cfg = Config::default();
        assert_eq!(cfg.layer_for("anything.exe"), 0);
    }

    #[test]
    fn first_matching_rule_wins() {
        let cfg = Config {
            rules: vec![rule("dup.exe", 2), rule("dup.exe", 7)],
            ..Config::default()
        };
        assert_eq!(cfg.layer_for("dup.exe"), 2);
    }

    #[test]
    fn config_survives_disk_round_trip() {
        let dir = std::env::temp_dir().join(format!("dls-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("config.cfg");

        let original = sample_config();
        write_config_to(&path, &original).unwrap();
        let loaded = read_config(&path).unwrap();

        assert_eq!(loaded.vendor_id, original.vendor_id);
        assert_eq!(loaded.product_id, original.product_id);
        assert_eq!(loaded.rules.len(), 2);
        assert_eq!(loaded.layer_for("code.exe"), 1);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn reading_missing_file_is_err() {
        let path = PathBuf::from("/nonexistent/dls/does-not-exist.cfg");
        assert!(read_config(&path).is_err());
    }
}
