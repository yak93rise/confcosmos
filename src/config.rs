use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Deserializer, Serialize};

/// A single symbolic entry under `[symbolic.symbolic_name]`
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SymbolicEntry {
    /// Full origin path (source path prefix joined with the user input)
    pub origin_path: String,
    /// Target path where the symlink will be created
    pub target_path: String,
    /// 新增时间：epoch 毫秒时间戳（兼容旧版 "YYYY-MM-DD HH:MM:SS mmm" 字符串）
    #[serde(default, deserialize_with = "de_millis")]
    pub add_datetime: i64,
    /// 修改时间：epoch 毫秒时间戳，0 表示尚未修改过
    #[serde(default)]
    pub edit_datetime: i64,
    /// Whether the symlink has already been generated
    #[serde(default)]
    pub already_generate: bool,
    /// 生成时间：epoch 毫秒时间戳，0 表示尚未生成
    #[serde(default)]
    pub generate_datetime: i64,
}

/// 反序列化时间戳：接受整数毫秒；也兼容旧的 "%Y-%m-%d %H:%M:%S %3f" 字符串。
fn de_millis<'de, D>(d: D) -> Result<i64, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Millis {
        Num(i64),
        Str(String),
    }
    Ok(match Millis::deserialize(d)? {
        Millis::Num(n) => n,
        Millis::Str(s) => parse_old_datetime(&s).unwrap_or(0),
    })
}

/// 把旧的 "YYYY-MM-DD HH:MM:SS mmm" 字符串解析为 epoch 毫秒。
fn parse_old_datetime(s: &str) -> Option<i64> {
    chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S %3f")
        .ok()
        .map(|dt| dt.and_utc().timestamp_millis())
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
    /// while there are no entries yet. 时间字段的值是 epoch 毫秒数字，
    /// 同时在对应字段上方写入人类可读的注释（不改动字段值本身）。
    pub fn save(&self, path: &Path) -> io::Result<()> {
        let mut doc: toml_edit::DocumentMut = toml::to_string_pretty(self)
            .map_err(io::Error::other)?
            .parse()
            .map_err(io::Error::other)?;

        add_datetime_comments(&mut doc, self);

        // 空配置时保持 [symbolic] 表存在
        if self.symbolic.is_empty() && doc.get("symbolic").is_none() {
            doc["symbolic"] = toml_edit::Item::Table(toml_edit::Table::new());
        }

        fs::write(path, doc.to_string())
    }
}

/// 在 toml 文档中，为每个条目的 datetime 字段上方添加人类可读的注释。
fn add_datetime_comments(doc: &mut toml_edit::DocumentMut, config: &Config) {
    let Some(symbolic) = doc
        .get_mut("symbolic")
        .and_then(toml_edit::Item::as_table_like_mut)
    else {
        return;
    };
    for name in config.symbolic.keys() {
        let Some(entry_table) = symbolic
            .get_mut(name)
            .and_then(toml_edit::Item::as_table_like_mut)
        else {
            continue;
        };
        let entry = &config.symbolic[name];
        add_time_comment(entry_table, "add_datetime", entry.add_datetime);
        add_time_comment(entry_table, "edit_datetime", entry.edit_datetime);
        add_time_comment(entry_table, "generate_datetime", entry.generate_datetime);
    }
}

/// 在 `key` 上方添加格式为 "YYYY-MM-DD HH:MM:SS mmm" 的注释；
/// millis <= 0（未设置）时不添加注释。
fn add_time_comment(table: &mut dyn toml_edit::TableLike, key: &str, millis: i64) {
    if millis <= 0 {
        return;
    }
    let Some(mut key) = table.key_mut(key) else {
        return;
    };
    let comment = format_comment_datetime(millis);
    key.leaf_decor_mut().set_prefix(format!("# {comment}\n"));
}

/// 把 epoch 毫秒格式化为 "YYYY-MM-DD HH:MM:SS mmm"（本地时间）。
fn format_comment_datetime(millis: i64) -> String {
    chrono::DateTime::from_timestamp_millis(millis)
        .map(|dt| {
            dt.with_timezone(&chrono::Local)
                .format("%Y-%m-%d %H:%M:%S %3f")
                .to_string()
        })
        .unwrap_or_default()
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
                add_datetime: 1705329022123,
                generate_datetime: 0,
                edit_datetime: 0,
                already_generate: false,
            },
        );
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("confcosmos.toml");
        cfg.save(&path).unwrap();
        let loaded: Config = toml::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(loaded.symbolic.len(), 1);
        assert_eq!(loaded.symbolic["vimrc"].origin_path, "/home/user/src/vimrc");
        assert_eq!(loaded.symbolic["vimrc"].add_datetime, 1705329022123);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn deserialize_legacy_add_datetime_string() {
        // 旧版配置 add_datetime 是字符串，应当兼容解析为毫秒时间戳
        let s = r#"
[symbolic.vimrc]
origin_path = "/a"
target_path = "/b"
add_datetime = "2024-01-15 14:30:22 123"
"#;
        let cfg: Config = toml::from_str(s).unwrap();
        let entry = &cfg.symbolic["vimrc"];
        assert!(
            entry.add_datetime > 0,
            "old string must be parsed to millis"
        );
        assert_eq!(entry.generate_datetime, 0);
        assert_eq!(entry.edit_datetime, 0);
        assert!(!entry.already_generate);
    }

    #[test]
    fn save_writes_ordered_fields_with_datetime_comments() {
        let dir = std::env::temp_dir()
            .join(format!("confcosmos-test-{}", std::process::id()))
            .join("comments");
        let mut cfg = Config {
            origin_path_prefix: "/home/user/src".into(),
            ..Default::default()
        };
        cfg.symbolic.insert(
            "vimrc".into(),
            SymbolicEntry {
                origin_path: "/home/user/src/vimrc".into(),
                target_path: "/home/user/.vimrc".into(),
                add_datetime: 1705329022123,
                edit_datetime: 1705331400000,
                already_generate: true,
                generate_datetime: 1705330800000,
            },
        );
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("confcosmos.toml");
        cfg.save(&path).unwrap();
        let s = std::fs::read_to_string(&path).unwrap();

        // 字段顺序：origin_path, target_path, add_datetime, edit_datetime,
        // already_generate, generate_datetime（带 " = " 避免匹配到 origin_path_prefix）
        let keys = [
            "origin_path = ",
            "target_path = ",
            "add_datetime = ",
            "edit_datetime = ",
            "already_generate = ",
            "generate_datetime = ",
        ];
        let mut last = 0;
        for k in keys {
            let pos = s.find(k).unwrap_or_else(|| panic!("missing {k} in: {s:?}"));
            assert!(pos > last, "{k} out of order in: {s:?}");
            last = pos;
        }

        // 时间字段上方有人类可读注释，且字段值仍是毫秒数字
        for millis in [1705329022123, 1705331400000, 1705330800000] {
            let expected = chrono::DateTime::from_timestamp_millis(millis)
                .unwrap()
                .with_timezone(&chrono::Local)
                .format("%Y-%m-%d %H:%M:%S %3f")
                .to_string();
            assert!(
                s.contains(&format!("# {expected}")),
                "missing comment {expected:?} in: {s:?}"
            );
            assert!(
                s.contains(&format!("{millis}\n")),
                "value {millis} missing: {s:?}"
            );
        }
        let _ = std::fs::remove_dir_all(&dir);
    }
}
