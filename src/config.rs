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
/// format = "binary"
///
/// # Controls whether Git-ignored entry detection is enabled in interactive mode.
/// # Supported values: true, false.
/// # If unset, behavior defaults to true.
/// # gitignore = true
///
/// # Controls whether cleanup heuristics are enabled in interactive mode.
/// # Supported values: true, false.
/// # If unset, behavior defaults to true.
/// # cleanup_heuristics = true
///
/// [keys]
/// esc_navigates_back = true
///
/// [notifications]
/// scan_finished = true
/// delete_finished = true
/// ```
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Byte count format to use when `--format` and `DUA_FORMAT` are not set.
    pub format: Option<crate::ByteFormat>,

    /// Keybinding-related settings.
    pub keys: KeysConfig,

    /// Interactive completion-notification settings.
    pub notifications: NotificationsConfig,

    /// Whether Git-ignored entry detection is enabled.
    ///
    /// Supported values: `true` and `false`.
    /// If unset, defaults to `true`.
    pub gitignore: Option<bool>,

    /// Whether cleanup heuristics are enabled.
    ///
    /// Supported values: `true` and `false`.
    /// If unset, defaults to `true`.
    pub cleanup_heuristics: Option<bool>,
}

/// Completion notifications emitted by interactive mode.
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct NotificationsConfig {
    /// Notify after initial scans and refreshes finish.
    pub scan_finished: bool,
    /// Notify after deletion or trash operations finish.
    pub delete_finished: bool,
}

impl Default for NotificationsConfig {
    fn default() -> Self {
        Self {
            scan_finished: true,
            delete_finished: true,
        }
    }
}

impl NotificationsConfig {
    /// Whether any notification needs terminal focus tracking.
    pub fn any_enabled(&self) -> bool {
        self.scan_finished || self.delete_finished
    }
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
    #[must_use]
    pub fn default_file_content() -> &'static str {
        concat!(
            "# dua-cli configuration\n",
            "#\n",
            "# Byte count format to use when --format and DUA_FORMAT are not set.\n",
            "# Supported values: metric, binary, bytes, gb, gib, mb, mib.\n",
            "# format = \"binary\"\n",
            "#\n",
            "# Controls whether Git-ignored entry detection is enabled in interactive mode.\n",
            "# Supported values: true, false.\n",
            "# If unset, behavior defaults to true.\n",
            "# gitignore = true\n",
            "#\n",
            "# Controls whether cleanup heuristics are enabled in interactive mode.\n",
            "# Supported values: true, false.\n",
            "# If unset, behavior defaults to true.\n",
            "# cleanup_heuristics = true\n",
            "#\n",
            "[keys]\n",
            "# If true, pressing <Esc> in the main pane ascends to the parent directory.\n",
            "# If false, <Esc> follows the default quit behavior.\n",
            "esc_navigates_back = true\n",
            "#\n",
            "[notifications]\n",
            "# Send terminal notifications when interactive operations finish while unfocused.\n",
            "scan_finished = true\n",
            "delete_finished = true\n",
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

#[cfg(test)]
mod tests {
    use super::Config;

    #[test]
    fn notifications_default_to_enabled_and_can_be_disabled() {
        let defaults: Config = toml::from_str("").expect("valid config");
        assert!(defaults.notifications.scan_finished);
        assert!(defaults.notifications.delete_finished);

        let configured: Config = toml::from_str(
            r"
            [notifications]
            scan_finished = false
            delete_finished = false
            ",
        )
        .expect("valid config");
        assert!(!configured.notifications.scan_finished);
        assert!(!configured.notifications.delete_finished);
    }

    #[test]
    fn notifications_are_enabled_if_any_notification_is_enabled() {
        let disabled: Config = toml::from_str(
            r"
            [notifications]
            scan_finished = false
            delete_finished = false
            ",
        )
        .expect("valid config");
        assert!(!disabled.notifications.any_enabled());

        let partly_enabled: Config = toml::from_str(
            r"
            [notifications]
            scan_finished = false
            ",
        )
        .expect("valid config");
        assert!(partly_enabled.notifications.any_enabled());
    }

    #[test]
    fn parses_configured_byte_format() {
        let config: Config = toml::from_str(
            r#"
            format = "mb"

            [keys]
            esc_navigates_back = false
            "#,
        )
        .expect("valid config");

        assert_eq!(config.format, Some(crate::ByteFormat::MB));
        assert!(!config.keys.esc_navigates_back);
    }

    #[test]
    fn parses_configured_gitignore() {
        let config: Config = toml::from_str(
            r#"
            format = "mb"
            gitignore = false

            [keys]
            esc_navigates_back = false
            "#,
        )
        .expect("valid config");

        assert_eq!(config.gitignore, Some(false));
    }

    #[test]
    fn gitignore_defaults_to_enabled() {
        let config: Config = toml::from_str(
            r#"
            format = "mb"

            [keys]
            esc_navigates_back = false
            "#,
        )
        .expect("valid config");

        assert_eq!(config.gitignore, None);
    }

    #[test]
    fn parses_configured_cleanup_heuristics() {
        let config: Config = toml::from_str(
            r#"
            format = "mb"
            cleanup_heuristics = false

            [keys]
            esc_navigates_back = false
            "#,
        )
        .expect("valid config");

        assert_eq!(config.cleanup_heuristics, Some(false));
    }

    #[test]
    fn cleanup_heuristics_defaults_to_enabled() {
        let config: Config = toml::from_str(
            r#"
            format = "mb"

            [keys]
            esc_navigates_back = false
            "#,
        )
        .expect("valid config");

        assert_eq!(config.cleanup_heuristics, None);
    }
}
