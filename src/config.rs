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
        // Use the OS-specific configuration directory (e.g. $XDG_CONFIG_HOME, %APPDATA%, or
        // ~/Library/Application Support) as provided by the `dirs` crate.
        let config_dir = dirs::config_dir()?;
        Some(config_dir.join("dua-cli").join("config.toml"))
    }
}
