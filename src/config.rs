use anyhow::{Context, Result};
use std::path::PathBuf;

#[derive(Debug, Default)]
pub struct Config {
    pub keys: KeysConfig,
}

#[derive(Debug, Default)]
pub struct KeysConfig {
    pub esc_navigates_back: bool,
}

impl Config {
    pub fn load() -> Result<Self> {
        let Some(path) = Self::path() else {
            return Ok(Config::default());
        };

        let contents = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Config::default()),
            Err(e) => {
                return Err(e)
                    .with_context(|| format!("Failed to read config at {}", path.display()));
            }
        };

        let table: toml::Table = contents
            .parse()
            .with_context(|| format!("Failed to parse config at {}", path.display()))?;

        let esc_navigates_back = table
            .get("keys")
            .and_then(|v| v.as_table())
            .and_then(|t| t.get("esc_navigates_back"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        Ok(Config {
            keys: KeysConfig { esc_navigates_back },
        })
    }

    fn path() -> Option<PathBuf> {
        // Use $XDG_CONFIG_HOME if set, otherwise ~/.config (standard for CLI tools on all platforms).
        let config_dir = std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .filter(|p| p.is_absolute())
            .or_else(|| dirs::home_dir().map(|h| h.join(".config")))?;
        Some(config_dir.join("dua-cli").join("config.toml"))
    }
}
