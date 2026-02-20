use anyhow::{Context, Result, anyhow};

use serde::Deserialize;

use std::path::PathBuf;

/// Runtime configuration used by interactive and CLI components.
///
/// The configuration file is optional. If it cannot be found, defaults are used.
/// See [`Config::load`] for details on fallback and error behavior.
///
/// Expected TOML structure:
///
/// ```toml
/// [keys]
/// esc_navigates_back = true
/// ```
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Keybinding-related settings.
    pub keys: KeysConfig,
}

/// Keyboard interaction settings.
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct KeysConfig {
    /// Changes `<Esc>` behavior in the interactive UI.
    ///
    /// If `true`, pressing `<Esc>` in the main pane ascends to the parent directory.
    /// If `false`, pressing `<Esc>` follows the default quit behavior, as if `q` was pressed.
    ///
    /// Default: `true`.
    #[serde(default = "default_esc_navigates_back")]
    pub esc_navigates_back: bool,
}

fn default_esc_navigates_back() -> bool {
    true
}

impl Default for KeysConfig {
    fn default() -> Self {
        Self {
            esc_navigates_back: default_esc_navigates_back(),
        }
    }
}

impl Config {
    /// Load configuration from disk.
    ///
    /// Behavior:
    /// - If no platform configuration directory is available, returns defaults.
    /// - If the config file does not exist, returns defaults.
    /// - If the config file exists but cannot be read, returns an error with path context.
    /// - If TOML parsing fails, returns an error with path context.
    ///
    /// Unknown keys are ignored. Missing supported keys fall back to defaults.
    pub fn load() -> Result<Self> {
        let Ok(path) = Self::path() else {
            log::info!("Configuration path couldn't be determined. Using defaults.");
            return Ok(Config::default());
        };

        let contents = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                log::info!(
                    "Configuration not loaded from {}: file not found. Using defaults.",
                    path.display()
                );
                return Ok(Config::default());
            }
            Err(e) => {
                return Err(e)
                    .with_context(|| format!("Failed to read config at {}", path.display()));
            }
        };

        toml::from_str(&contents)
            .with_context(|| format!("Failed to parse config at {}", path.display()))
    }

    /// Default TOML content used when initializing a new configuration file.
    pub fn default_file_content() -> &'static str {
        concat!(
            "# dua-cli configuration\n",
            "#\n",
            "[keys]\n",
            "# If true, pressing <Esc> in the main pane ascends to the parent directory.\n",
            "# If false, <Esc> follows the default quit behavior.\n",
            "esc_navigates_back = true\n",
        )
    }

    /// Return the expected configuration file location for the current platform.
    ///
    /// The path is:
    /// - Linux/Unix: `$XDG_CONFIG_HOME/dua-cli/config.toml` (or equivalent fallback)
    /// - Windows: `%APPDATA%\\dua-cli\\config.toml`
    /// - macOS: `~/Library/Application Support/dua-cli/config.toml`
    ///
    /// Returns an error if the platform config directory cannot be determined.
    pub fn path() -> Result<PathBuf> {
        // Use the OS-specific configuration directory (e.g. $XDG_CONFIG_HOME, %APPDATA%, or
        // ~/Library/Application Support) as provided by the `dirs` crate.
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow!("platform config directory is unavailable"))?;
        Ok(config_dir.join("dua-cli").join("config.toml"))
    }
}
