use anyhow::{Context, Result, anyhow};

use serde::{Deserialize, Deserializer, de};

use std::{fmt, path::PathBuf, str::FromStr};

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
/// sort_by_name = "ctrl+n"
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

/// One or more keys that invoke an interactive action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyBindings(Vec<KeyBinding>);

const UNMAPPED_KEY: &str = "<unmapped>";

impl KeyBindings {
    fn defaults(bindings: &[&str]) -> Self {
        Self(
            bindings
                .iter()
                .map(|binding| binding.parse().expect("valid built-in keybinding"))
                .collect(),
        )
    }

    /// Return whether `key` invokes this action.
    #[cfg(feature = "tui-crossplatform")]
    #[must_use]
    pub fn matches(&self, key: crossterm::event::KeyEvent) -> bool {
        self.0.iter().any(|binding| binding.matches(key))
    }

    /// Render the first configured key for compact interface hints.
    #[must_use]
    pub fn primary(&self) -> String {
        self.0
            .first()
            .map_or_else(|| UNMAPPED_KEY.into(), ToString::to_string)
    }

    /// Return whether this action has no configured keys.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl fmt::Display for KeyBindings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.is_empty() {
            return f.write_str(UNMAPPED_KEY);
        }
        for (index, binding) in self.0.iter().enumerate() {
            if index != 0 {
                f.write_str("/")?;
            }
            binding.fmt(f)?;
        }
        Ok(())
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum KeyBindingsRepr {
    One(String),
    Many(Vec<String>),
}

impl<'de> Deserialize<'de> for KeyBindings {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        match KeyBindingsRepr::deserialize(deserializer)? {
            KeyBindingsRepr::One(binding) => vec![binding],
            KeyBindingsRepr::Many(bindings) => bindings,
        }
        .into_iter()
        .map(|binding| binding.parse().map_err(de::Error::custom))
        .collect::<Result<Vec<_>, _>>()
        .map(Self)
    }
}

/// One key and its optional modifiers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyBinding {
    code: Key,
    modifiers: Vec<Modifier>,
}

impl FromStr for KeyBinding {
    type Err = String;

    fn from_str(binding: &str) -> Result<Self, Self::Err> {
        if binding.is_empty() {
            return Err("keybinding cannot be empty".into());
        }

        let mut parts = if binding == "+" {
            vec![binding]
        } else {
            binding.split('+').map(str::trim).collect::<Vec<_>>()
        };
        let key = parts
            .pop()
            .filter(|key| !key.is_empty())
            .ok_or_else(|| format!("keybinding '{binding}' has no key"))?;
        let mut modifiers = Vec::new();
        for modifier in parts {
            let modifier = match modifier.to_ascii_lowercase().as_str() {
                "ctrl" | "control" => Modifier::Control,
                "alt" => Modifier::Alt,
                "shift" => Modifier::Shift,
                _ => {
                    return Err(format!(
                        "unknown modifier '{modifier}' in keybinding '{binding}'"
                    ));
                }
            };
            if !modifiers.contains(&modifier) {
                modifiers.push(modifier);
            }
        }

        let code = Key::from_str(key).map_err(|err| format!("{err} in keybinding '{binding}'"))?;
        Ok(Self { code, modifiers })
    }
}

impl fmt::Display for KeyBinding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for modifier in &self.modifiers {
            write!(f, "{modifier} + ")?;
        }
        self.code.fmt(f)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Key {
    Char(char),
    Backspace,
    Enter,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
    Tab,
    BackTab,
    Delete,
    Insert,
    Function(u8),
    Null,
    Esc,
    CapsLock,
    ScrollLock,
    NumLock,
    PrintScreen,
    Pause,
    Menu,
    KeypadBegin,
}

impl FromStr for Key {
    type Err = String;

    fn from_str(key: &str) -> Result<Self, Self::Err> {
        if key.chars().count() == 1 {
            return Ok(Self::Char(key.chars().next().expect("one character")));
        }

        let normalized = key.to_ascii_lowercase();
        let key = match normalized.as_str() {
            "space" => Self::Char(' '),
            "plus" => Self::Char('+'),
            "backspace" => Self::Backspace,
            "enter" | "return" => Self::Enter,
            "left" => Self::Left,
            "right" => Self::Right,
            "up" => Self::Up,
            "down" => Self::Down,
            "home" => Self::Home,
            "end" => Self::End,
            "page-up" | "pageup" => Self::PageUp,
            "page-down" | "pagedown" => Self::PageDown,
            "tab" => Self::Tab,
            "back-tab" | "backtab" => Self::BackTab,
            "delete" | "del" => Self::Delete,
            "insert" => Self::Insert,
            "null" => Self::Null,
            "esc" | "escape" => Self::Esc,
            "caps-lock" => Self::CapsLock,
            "scroll-lock" => Self::ScrollLock,
            "num-lock" => Self::NumLock,
            "print-screen" => Self::PrintScreen,
            "pause" => Self::Pause,
            "menu" => Self::Menu,
            "keypad-begin" => Self::KeypadBegin,
            function if function.starts_with('f') => {
                let number = function[1..]
                    .parse()
                    .map_err(|_| format!("unknown key '{key}'"))?;
                if number == 0 {
                    return Err(format!("unknown key '{key}'"));
                }
                Self::Function(number)
            }
            _ => return Err(format!("unknown key '{key}'")),
        };
        Ok(key)
    }
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Char(' ') => f.write_str("<Space>"),
            Self::Char(character) => character.fmt(f),
            Self::Backspace => f.write_str("<Backspace>"),
            Self::Enter => f.write_str("<Enter>"),
            Self::Left => f.write_str("<Left>"),
            Self::Right => f.write_str("<Right>"),
            Self::Up => f.write_str("<Up>"),
            Self::Down => f.write_str("<Down>"),
            Self::Home => f.write_str("<Home>"),
            Self::End => f.write_str("<End>"),
            Self::PageUp => f.write_str("<Page Up>"),
            Self::PageDown => f.write_str("<Page Down>"),
            Self::Tab => f.write_str("<Tab>"),
            Self::BackTab => f.write_str("<Back Tab>"),
            Self::Delete => f.write_str("<Delete>"),
            Self::Insert => f.write_str("<Insert>"),
            Self::Function(number) => write!(f, "<F{number}>"),
            Self::Null => f.write_str("<Null>"),
            Self::Esc => f.write_str("<Esc>"),
            Self::CapsLock => f.write_str("<Caps Lock>"),
            Self::ScrollLock => f.write_str("<Scroll Lock>"),
            Self::NumLock => f.write_str("<Num Lock>"),
            Self::PrintScreen => f.write_str("<Print Screen>"),
            Self::Pause => f.write_str("<Pause>"),
            Self::Menu => f.write_str("<Menu>"),
            Self::KeypadBegin => f.write_str("<Keypad Begin>"),
        }
    }
}

#[cfg(feature = "tui-crossplatform")]
impl Key {
    fn to_crossterm(self) -> crossterm::event::KeyCode {
        use crossterm::event::KeyCode;

        match self {
            Self::Char(character) => KeyCode::Char(character),
            Self::Backspace => KeyCode::Backspace,
            Self::Enter => KeyCode::Enter,
            Self::Left => KeyCode::Left,
            Self::Right => KeyCode::Right,
            Self::Up => KeyCode::Up,
            Self::Down => KeyCode::Down,
            Self::Home => KeyCode::Home,
            Self::End => KeyCode::End,
            Self::PageUp => KeyCode::PageUp,
            Self::PageDown => KeyCode::PageDown,
            Self::Tab => KeyCode::Tab,
            Self::BackTab => KeyCode::BackTab,
            Self::Delete => KeyCode::Delete,
            Self::Insert => KeyCode::Insert,
            Self::Function(number) => KeyCode::F(number),
            Self::Null => KeyCode::Null,
            Self::Esc => KeyCode::Esc,
            Self::CapsLock => KeyCode::CapsLock,
            Self::ScrollLock => KeyCode::ScrollLock,
            Self::NumLock => KeyCode::NumLock,
            Self::PrintScreen => KeyCode::PrintScreen,
            Self::Pause => KeyCode::Pause,
            Self::Menu => KeyCode::Menu,
            Self::KeypadBegin => KeyCode::KeypadBegin,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Modifier {
    Control,
    Alt,
    Shift,
}

impl fmt::Display for Modifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Control => "Ctrl",
            Self::Alt => "Alt",
            Self::Shift => "Shift",
        })
    }
}

#[cfg(feature = "tui-crossplatform")]
impl KeyBinding {
    /// Convert this binding into the terminal event it describes.
    #[must_use]
    pub fn to_event(&self) -> crossterm::event::KeyEvent {
        use crossterm::event::{KeyEvent, KeyModifiers};

        let modifiers = self
            .modifiers
            .iter()
            .fold(KeyModifiers::NONE, |modifiers, modifier| {
                modifiers
                    | match modifier {
                        Modifier::Control => KeyModifiers::CONTROL,
                        Modifier::Alt => KeyModifiers::ALT,
                        Modifier::Shift => KeyModifiers::SHIFT,
                    }
            });
        KeyEvent::new(self.code.to_crossterm(), modifiers)
    }

    fn matches(&self, key: crossterm::event::KeyEvent) -> bool {
        use crossterm::event::{KeyEvent, KeyModifiers};

        let mut key_modifiers = key.modifiers;
        if !self.modifiers.contains(&Modifier::Shift)
            && matches!(self.code, Key::Char(character) if !character.is_alphanumeric())
        {
            key_modifiers.remove(KeyModifiers::SHIFT);
        }
        self.to_event() == KeyEvent::new(key.code, key_modifiers)
    }
}

/// Keyboard interaction settings.
#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct KeysConfig {
    /// Changes the configured close-pane key behavior in the interactive UI.
    ///
    /// If `true`, pressing it in the main pane ascends to the parent directory.
    /// If `false`, it follows the quit behavior.
    ///
    /// Default: `true`.
    #[serde(default = "default_esc_navigates_back")]
    pub esc_navigates_back: bool,

    /// Close the focused pane.
    pub close_pane: KeyBindings,
    /// Close a pane or quit from the main pane.
    pub quit: KeyBindings,
    /// Quit immediately without confirmation.
    pub quit_immediately: KeyBindings,
    /// Suspend the process and return control to the shell on Unix.
    pub suspend: KeyBindings,
    /// Clear and repaint the screen.
    pub repaint: KeyBindings,
    /// Move focus to the next open pane.
    pub cycle_panes: KeyBindings,
    /// Show or hide help.
    pub toggle_help: KeyBindings,
    /// Open the glob-search pane.
    pub open_search: KeyBindings,
    /// Move down one item.
    pub move_down: KeyBindings,
    /// Move up one item.
    pub move_up: KeyBindings,
    /// Move down one page.
    pub page_down: KeyBindings,
    /// Move up one page.
    pub page_up: KeyBindings,
    /// Move to the first item.
    pub move_to_top: KeyBindings,
    /// Move to the last item.
    pub move_to_bottom: KeyBindings,
    /// Enter the selected directory.
    pub descend: KeyBindings,
    /// Ascend to the parent directory.
    pub ascend: KeyBindings,
    /// Scan the directory above the current traversal root.
    pub scan_parent: KeyBindings,
    /// Sort by size.
    pub sort_by_size: KeyBindings,
    /// Sort by modification time.
    pub sort_by_mtime: KeyBindings,
    /// Cycle the modification-time mode or column.
    pub cycle_mtime_mode: KeyBindings,
    /// Sort by item count.
    pub sort_by_count: KeyBindings,
    /// Show or hide the item-count column.
    pub toggle_count_column: KeyBindings,
    /// Sort by name.
    pub sort_by_name: KeyBindings,
    /// Cycle byte visualizations.
    pub cycle_visualization: KeyBindings,
    /// Open the selected entry externally.
    pub open_entry: KeyBindings,
    /// Toggle the selected entry's mark.
    pub toggle_mark: KeyBindings,
    /// Mark the selected entry for deletion.
    pub mark_for_deletion: KeyBindings,
    /// Toggle the selected entry's mark and move down.
    pub toggle_mark_and_move_down: KeyBindings,
    /// Toggle all visible entry marks.
    pub toggle_all: KeyBindings,
    /// Toggle cleanup-candidate detection.
    pub toggle_cleanup: KeyBindings,
    /// Mark cleanup candidates.
    pub mark_cleanup: KeyBindings,
    /// Toggle Git-ignore detection.
    pub toggle_gitignore: KeyBindings,
    /// Mark Git-ignored entries.
    pub mark_gitignore: KeyBindings,
    /// Refresh the selected entry.
    pub refresh_selected: KeyBindings,
    /// Refresh the current view.
    pub refresh_all: KeyBindings,
    /// Remove the selected mark.
    pub remove_mark: KeyBindings,
    /// Remove all marks.
    pub remove_all_marks: KeyBindings,
    /// Permanently delete marked entries.
    pub delete_marked: KeyBindings,
    /// Move marked entries to the trash.
    pub trash_marked: KeyBindings,
    /// Submit the glob search.
    pub search_confirm: KeyBindings,
    /// Toggle glob-search case sensitivity.
    pub search_toggle_case: KeyBindings,
    /// Delete the preceding glob-search character.
    pub search_backspace: KeyBindings,
    /// Move the glob-search cursor left.
    pub search_left: KeyBindings,
    /// Move the glob-search cursor right.
    pub search_right: KeyBindings,
}

fn default_esc_navigates_back() -> bool {
    true
}

impl Default for KeysConfig {
    fn default() -> Self {
        Self {
            esc_navigates_back: default_esc_navigates_back(),
            close_pane: KeyBindings::defaults(&["esc"]),
            quit: KeyBindings::defaults(&["q"]),
            quit_immediately: KeyBindings::defaults(&["ctrl+c"]),
            suspend: KeyBindings::defaults(&["ctrl+z"]),
            repaint: KeyBindings::defaults(&["ctrl+l"]),
            cycle_panes: KeyBindings::defaults(&["tab"]),
            toggle_help: KeyBindings::defaults(&["?"]),
            open_search: KeyBindings::defaults(&["/"]),
            move_down: KeyBindings::defaults(&["j", "down"]),
            move_up: KeyBindings::defaults(&["k", "up"]),
            page_down: KeyBindings::defaults(&["ctrl+d", "page-down"]),
            page_up: KeyBindings::defaults(&["ctrl+u", "page-up"]),
            move_to_top: KeyBindings::defaults(&["H", "home"]),
            move_to_bottom: KeyBindings::defaults(&["G", "end"]),
            descend: KeyBindings::defaults(&["o", "l", "enter", "right"]),
            ascend: KeyBindings::defaults(&["u", "h", "backspace", "left"]),
            scan_parent: KeyBindings::defaults(&["U"]),
            sort_by_size: KeyBindings::defaults(&["s"]),
            sort_by_mtime: KeyBindings::defaults(&["m"]),
            cycle_mtime_mode: KeyBindings::defaults(&["M"]),
            sort_by_count: KeyBindings::defaults(&["c"]),
            toggle_count_column: KeyBindings::defaults(&["C"]),
            sort_by_name: KeyBindings::defaults(&["n"]),
            cycle_visualization: KeyBindings::defaults(&["g", "S"]),
            open_entry: KeyBindings::defaults(&["O"]),
            toggle_mark: KeyBindings::defaults(&["space"]),
            mark_for_deletion: KeyBindings::defaults(&["x"]),
            toggle_mark_and_move_down: KeyBindings::defaults(&["d"]),
            toggle_all: KeyBindings::defaults(&["a"]),
            toggle_cleanup: KeyBindings::defaults(&["t"]),
            mark_cleanup: KeyBindings::defaults(&["X"]),
            toggle_gitignore: KeyBindings::defaults(&["i"]),
            mark_gitignore: KeyBindings::defaults(&["I"]),
            refresh_selected: KeyBindings::defaults(&["r"]),
            refresh_all: KeyBindings::defaults(&["R"]),
            remove_mark: KeyBindings::defaults(&["x", "d", "space"]),
            remove_all_marks: KeyBindings::defaults(&["a"]),
            delete_marked: KeyBindings::defaults(&["ctrl+r"]),
            trash_marked: KeyBindings::defaults(&["ctrl+t"]),
            search_confirm: KeyBindings::defaults(&["enter"]),
            search_toggle_case: KeyBindings::defaults(&["ctrl+f"]),
            search_backspace: KeyBindings::defaults(&["backspace"]),
            search_left: KeyBindings::defaults(&["left"]),
            search_right: KeyBindings::defaults(&["right"]),
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
            "# If true, close_pane keys ascend from the main pane.\n",
            "# If false, close_pane keys follow the quit behavior.\n",
            "esc_navigates_back = true\n",
            "#\n",
            "# Use a string for one binding or an array for aliases. Uncomment to replace the defaults.\n",
            "# Modifiers: ctrl, alt, shift. Named keys include\n",
            "# esc, enter, space, tab, backspace, arrows, home, end, page-up, page-down, and F1-F255.\n",
            "# Character keys are case-sensitive; use [] to disable an action.\n",
            "#\n",
            "# Pane and application control.\n",
            "# close_pane = \"esc\"\n",
            "# quit = \"q\"\n",
            "# quit_immediately = \"ctrl+c\"\n",
            "# suspend = \"ctrl+z\" # Unix only.\n",
            "# repaint = \"ctrl+l\"\n",
            "# cycle_panes = \"tab\"\n",
            "# toggle_help = \"?\"\n",
            "# open_search = \"/\"\n",
            "#\n",
            "# Navigation.\n",
            "# move_down = [\"j\", \"down\"]\n",
            "# move_up = [\"k\", \"up\"]\n",
            "# page_down = [\"ctrl+d\", \"page-down\"]\n",
            "# page_up = [\"ctrl+u\", \"page-up\"]\n",
            "# move_to_top = [\"H\", \"home\"]\n",
            "# move_to_bottom = [\"G\", \"end\"]\n",
            "# descend = [\"o\", \"l\", \"enter\", \"right\"]\n",
            "# ascend = [\"u\", \"h\", \"backspace\", \"left\"]\n",
            "# scan_parent = \"U\"\n",
            "#\n",
            "# Display.\n",
            "# sort_by_size = \"s\"\n",
            "# sort_by_mtime = \"m\"\n",
            "# cycle_mtime_mode = \"M\"\n",
            "# sort_by_count = \"c\"\n",
            "# toggle_count_column = \"C\"\n",
            "# sort_by_name = \"n\"\n",
            "# cycle_visualization = [\"g\", \"S\"]\n",
            "#\n",
            "# Entry actions.\n",
            "# open_entry = \"O\"\n",
            "# toggle_mark = \"space\"\n",
            "# mark_for_deletion = \"x\"\n",
            "# toggle_mark_and_move_down = \"d\"\n",
            "# toggle_all = \"a\"\n",
            "# toggle_cleanup = \"t\"\n",
            "# mark_cleanup = \"X\"\n",
            "# toggle_gitignore = \"i\"\n",
            "# mark_gitignore = \"I\"\n",
            "# refresh_selected = \"r\"\n",
            "# refresh_all = \"R\"\n",
            "#\n",
            "# Marked-items pane.\n",
            "# remove_mark = [\"x\", \"d\", \"space\"]\n",
            "# remove_all_marks = \"a\"\n",
            "# delete_marked = \"ctrl+r\"\n",
            "# trash_marked = \"ctrl+t\"\n",
            "#\n",
            "# Search pane.\n",
            "# search_confirm = \"enter\"\n",
            "# search_toggle_case = \"ctrl+f\"\n",
            "# search_backspace = \"backspace\"\n",
            "# search_left = \"left\"\n",
            "# search_right = \"right\"\n",
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
    fn keybindings_keep_current_defaults_and_are_documented() {
        let expected_actions = [
            "close_pane",
            "quit",
            "quit_immediately",
            "suspend",
            "repaint",
            "cycle_panes",
            "toggle_help",
            "open_search",
            "move_down",
            "move_up",
            "page_down",
            "page_up",
            "move_to_top",
            "move_to_bottom",
            "descend",
            "ascend",
            "scan_parent",
            "sort_by_size",
            "sort_by_mtime",
            "cycle_mtime_mode",
            "sort_by_count",
            "toggle_count_column",
            "sort_by_name",
            "cycle_visualization",
            "open_entry",
            "toggle_mark",
            "mark_for_deletion",
            "toggle_mark_and_move_down",
            "toggle_all",
            "toggle_cleanup",
            "mark_cleanup",
            "toggle_gitignore",
            "mark_gitignore",
            "refresh_selected",
            "refresh_all",
            "remove_mark",
            "remove_all_marks",
            "delete_marked",
            "trash_marked",
            "search_confirm",
            "search_toggle_case",
            "search_backspace",
            "search_left",
            "search_right",
        ];
        let defaults = Config::default_file_content();

        for action in expected_actions {
            assert!(
                defaults
                    .lines()
                    .any(|line| line.starts_with(&format!("# {action} = "))),
                "missing default template for keys.{action}"
            );
        }

        let configured: Config = toml::from_str(&format!(
            "[keys]\n{}",
            defaults
                .lines()
                .skip_while(|line| *line != "[keys]")
                .skip(1)
                .take_while(|line| !line.starts_with('['))
                .filter_map(|line| line.strip_prefix("# ").filter(|line| line.contains(" = ")))
                .collect::<Vec<_>>()
                .join("\n")
        ))
        .expect("documented default keybindings are valid");
        assert_eq!(configured.keys, Config::default().keys);
    }

    #[test]
    fn invalid_keybinding_is_rejected() {
        let err = toml::from_str::<Config>(
            r#"
            [keys]
            quit = ["ctrl+definitely-not-a-key"]
            "#,
        )
        .expect_err("unknown keys must not be silently ignored");
        assert!(err.to_string().contains("unknown key"));
    }

    #[test]
    fn keybindings_accept_strings_arrays_and_empty_arrays() {
        let configured: Config = toml::from_str(
            r#"
            [keys]
            quit = "ctrl+q"
            move_down = ["j", "down"]
            open_search = []
            "#,
        )
        .expect("valid config");

        assert_eq!(configured.keys.quit.to_string(), "Ctrl + q");
        assert_eq!(configured.keys.move_down.to_string(), "j/<Down>");
        assert_eq!(configured.keys.open_search.to_string(), "<unmapped>");
        assert_eq!(configured.keys.open_search.primary(), "<unmapped>");
    }

    #[cfg(feature = "tui-crossplatform")]
    #[test]
    fn keybindings_distinguish_modifiers_and_normalize_shifted_characters() {
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

        let config: Config = toml::from_str(
            r#"
            [keys]
            toggle_help = ["x"]
            quit_immediately = ["ctrl+x"]
            sort_by_name = ["shift+n"]
            "#,
        )
        .expect("valid config");
        let plain = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE);
        let control = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::CONTROL);

        assert!(config.keys.toggle_help.matches(plain));
        assert!(!config.keys.toggle_help.matches(control));
        assert!(config.keys.quit_immediately.matches(control));
        assert!(
            config
                .keys
                .suspend
                .matches(KeyEvent::new(KeyCode::Char('z'), KeyModifiers::CONTROL,))
        );
        assert!(
            config
                .keys
                .sort_by_name
                .matches(KeyEvent::new(KeyCode::Char('N'), KeyModifiers::SHIFT))
        );
        assert!(
            config
                .keys
                .sort_by_name
                .matches(KeyEvent::new(KeyCode::Char('n'), KeyModifiers::SHIFT))
        );
    }

    #[cfg(feature = "tui-crossplatform")]
    #[test]
    fn unmodified_punctuation_accepts_an_implicit_shift_modifier() {
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

        let keys = Config::default().keys;
        assert!(
            keys.toggle_help
                .matches(KeyEvent::new(KeyCode::Char('?'), KeyModifiers::SHIFT))
        );
        assert!(!keys.toggle_help.matches(KeyEvent::new(
            KeyCode::Char('?'),
            KeyModifiers::SHIFT | KeyModifiers::CONTROL,
        )));
    }

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
