//! Heuristics for cleaning up common project directories.
use gix_glob::Pattern;
use serde::Deserialize;
use std::ffi::OsStr;
use std::sync::OnceLock;

#[cfg(any(windows, target_os = "macos"))]
const IGNORE_CASE: bool = true;
#[cfg(not(any(windows, target_os = "macos")))]
const IGNORE_CASE: bool = false;

#[derive(Debug, Deserialize, Clone)]
/// Configuration for heuristics.
pub struct HeuristicConfig {
    /// List of heuristics.
    pub heuristics: Vec<Heuristic>,
}

#[derive(Debug, Deserialize, Clone)]
/// A heuristic for cleaning up directories.
pub struct Heuristic {
    /// Name of the heuristic.
    pub name: String,
    /// Description of the heuristic.
    pub description: String,
    #[serde(default = "default_true")]
    /// Whether the heuristic is enabled.
    pub enabled: bool,
    #[serde(rename = "match")]
    /// Rules to match the heuristic.
    pub match_rules: Vec<String>,
    /// Patterns to clean up.
    pub patterns: Vec<String>,
}

fn default_true() -> bool {
    true
}

impl Heuristic {
    /// Check if the heuristic matches the given directory entries.
    pub fn matches<'a>(&self, entries: impl Iterator<Item = (bool, &'a OsStr)> + Clone) -> bool {
        if !self.enabled {
            return false;
        }

        for rule_group in &self.match_rules {
            let mut group_matched = false;
            for rule in rule_group.split('|').map(|s| s.trim()) {
                if rule.starts_with("-f ") {
                    let file_name = &rule[3..];
                    // Support glob in file name
                    if file_name.contains('*') || file_name.contains('?') {
                        if let Some(pattern) = Pattern::from_bytes(file_name.as_bytes()) {
                            if entries.clone().any(|(is_dir, name)| {
                                !is_dir && pattern.matches(
                                    bstr::BStr::new(name.as_encoded_bytes()),
                                    if IGNORE_CASE {
                                        gix_glob::wildmatch::Mode::IGNORE_CASE
                                    } else {
                                        gix_glob::wildmatch::Mode::empty()
                                    },
                                )
                            }) {
                                group_matched = true;
                                break;
                            }
                        }
                    } else if entries.clone().any(|(is_dir, name)| {
                        !is_dir && if IGNORE_CASE {
                            name.to_string_lossy().eq_ignore_ascii_case(file_name)
                        } else {
                            name == file_name
                        }
                    }) {
                        group_matched = true;
                        break;
                    }
                } else if rule.starts_with("-d ") {
                    let dir_name = &rule[3..];
                    if entries.clone().any(|(is_dir, name)| {
                        is_dir && if IGNORE_CASE {
                            name.to_string_lossy().eq_ignore_ascii_case(dir_name)
                        } else {
                            name == dir_name
                        }
                    }) {
                        group_matched = true;
                        break;
                    }
                }
            }
            if !group_matched {
                return false;
            }
        }
        true
    }
}

/// Load all heuristics.
pub fn load_heuristics() -> &'static [Heuristic] {
    static HEURISTICS: OnceLock<Vec<Heuristic>> = OnceLock::new();
    HEURISTICS.get_or_init(|| {
        let mut all = Vec::new();
        let configs = include!(concat!(env!("OUT_DIR"), "/heuristics_includes.rs"));

        for config_str in configs {
            if let Ok(config) = toml::from_str::<HeuristicConfig>(config_str) {
                all.extend(config.heuristics);
            }
        }
        all
    })
}
