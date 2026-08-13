use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// A single symbolic entry under `[symbolic.symbolic_name]`
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SymbolicEntry {
    /// Full origin path (source path prefix joined with the user input)
    pub origin_path: String,
    /// Target path where the symlink will be created
    pub target_path: String,
    /// Human readable millisecond timestamp, e.g. "2024-01-15 14:30:22 123"
    pub add_datetime: String,
    /// Whether the symlink has already been generated
    #[serde(default)]
    pub already_generate: bool,
}

/// Root config of `confcosmos.toml`
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Config {
    /// Source path prefix, all origin paths are joined with this
    pub origin_path_prefix: String,
    /// `[symbolic]` nested table
    #[serde(default)]
    pub symbolic: BTreeMap<String, SymbolicEntry>,
}

impl Config {
    /// Load the config from `~/.config/confcosmos/confcosmos.toml`.
    ///
    /// - Creates the `confcosmos` folder and an empty `confcosmos.toml` if the
    ///   folder / file does not exist yet.
    /// - Returns `(config, config_path, needs_setup)` where `needs_setup` is
    ///   `true` when the file was just created and the user still has to input
    ///   the source path prefix.
    pub fn load() -> io::Result<(Config, PathBuf, bool)> {
        let home = std::env::var("HOME").unwrap_or_default();
        let dir = Path::new(&home).join(".config").join("confcosmos");
        fs::create_dir_all(&dir)?;

        let path = dir.join("confcosmos.toml");
        if path.exists() {
            let content = fs::read_to_string(&path)?;
            let config: Config = toml::from_str(&content).unwrap_or_default();
            Ok((config, path, false))
        } else {
            // Create an empty toml config file, the user will be asked to
            // input the origin_path_prefix on the setup page afterwards.
            fs::write(&path, "[symbolic]\n")?;
            Ok((Config::default(), path, true))
        }
    }

    /// Save the config, always keeping the empty `[symbolic]` table around
    /// while there are no entries yet.
    pub fn save(&self, path: &Path) -> io::Result<()> {
        let mut s = toml::to_string_pretty(self).map_err(io::Error::other)?;
        if self.symbolic.is_empty() && !s.contains("[symbolic]") {
            s.push_str("\n[symbolic]\n");
        }
        fs::write(path, s)
    }
}

/// Expand a leading `~` / `$HOME` / `${HOME}` to the home directory.
pub fn expand_path(input: &str, home: &str) -> String {
    if input == "~" {
        return home.to_string();
    }
    if let Some(rest) = input.strip_prefix("~/") {
        return format!("{}/{}", home, rest);
    }
    for (prefix, rest) in [
        ("$HOME", input.strip_prefix("$HOME")),
        ("${HOME}", input.strip_prefix("${HOME}")),
    ] {
        if let Some(rest) = rest {
            let _ = prefix;
            if rest.is_empty() {
                return home.to_string();
            }
            return format!("{}{}", home, rest);
        }
    }
    input.to_string()
}

/// Join the source path prefix with the user input origin path.
pub fn join_origin_path(prefix: &str, input: &str) -> String {
    if prefix.is_empty() {
        return input.to_string();
    }
    if prefix.ends_with('/') || input.starts_with('/') {
        format!("{}{}", prefix, input)
    } else {
        format!("{}/{}", prefix, input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expand_tilde() {
        let home = "/home/user";
        assert_eq!(expand_path("~", home), "/home/user");
        assert_eq!(expand_path("~/dotfiles", home), "/home/user/dotfiles");
        assert_eq!(expand_path("$HOME", home), "/home/user");
        assert_eq!(expand_path("$HOME/.config", home), "/home/user/.config");
        assert_eq!(expand_path("${HOME}/x", home), "/home/user/x");
        assert_eq!(expand_path("/etc", home), "/etc");
    }

    #[test]
    fn join_origin() {
        assert_eq!(
            join_origin_path("/home/user/src", "a/b"),
            "/home/user/src/a/b"
        );
        assert_eq!(
            join_origin_path("/home/user/src/", "a/b"),
            "/home/user/src/a/b"
        );
        assert_eq!(join_origin_path("", "a/b"), "a/b");
        assert_eq!(
            join_origin_path("/home/user/src", "/abs/path"),
            "/home/user/src/abs/path"
        );
    }

    #[test]
    fn save_keeps_symbolic_table() {
        let dir = std::env::temp_dir()
            .join(format!("confcosmos-test-{}", std::process::id()))
            .join("save_keeps");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("confcosmos.toml");

        let cfg = Config::default();
        cfg.save(&path).unwrap();
        let s = std::fs::read_to_string(&path).unwrap();
        assert!(
            s.contains("[symbolic]"),
            "empty [symbolic] table must be present: {s:?}"
        );

        let loaded: Config = toml::from_str(&s).unwrap();
        assert!(loaded.symbolic.is_empty());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn roundtrip_entry() {
        let dir = std::env::temp_dir()
            .join(format!("confcosmos-test-{}", std::process::id()))
            .join("roundtrip");
        let mut cfg = Config {
            origin_path_prefix: "/home/user/src".into(),
            ..Default::default()
        };
        cfg.symbolic.insert(
            "vimrc".into(),
            SymbolicEntry {
                origin_path: "/home/user/src/vimrc".into(),
                target_path: "/home/user/.vimrc".into(),
                add_datetime: "2024-01-15 14:30:22 123".into(),
                already_generate: false,
            },
        );
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("confcosmos.toml");
        cfg.save(&path).unwrap();
        let loaded: Config = toml::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(loaded.symbolic.len(), 1);
        assert_eq!(loaded.symbolic["vimrc"].origin_path, "/home/user/src/vimrc");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
