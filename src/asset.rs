use std::collections::HashMap;

use bevy::asset::{AssetLoader, LoadContext, io::Reader};
use bevy::prelude::{Asset, TypePath};
use serde::{Deserialize, Serialize};

/// A single locale's translation data, loaded as a Bevy Asset.
#[derive(Debug, Asset, TypePath, Clone)]
pub struct I18nAsset {
    /// Flat key -> translation string map.
    /// Keys use dot-notation: "game.title", "player.greeting"
    entries: HashMap<String, String>,
}

impl I18nAsset {
    /// Look up a translation by key.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.entries.get(key).map(|s| s.as_str())
    }
}

/// Settings for the i18n asset loader (empty - no config needed).
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct I18nLoaderSettings;

/// AssetLoader that parses translation files into I18nAsset.
/// Supports YAML (default) and .po (feature: po).
#[derive(Default, bevy::prelude::TypePath)]
pub struct I18nLoader;

impl AssetLoader for I18nLoader {
    type Asset = I18nAsset;
    type Settings = I18nLoaderSettings;
    type Error = Box<dyn std::error::Error + Send + Sync>;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &I18nLoaderSettings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<I18nAsset, Self::Error> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf).await?;
        let content = String::from_utf8_lossy(&buf);

        let extension = load_context
            .path()
            .path()
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        match extension {
            #[cfg(feature = "yaml")]
            "yaml" | "yml" => {
                let yaml_value: serde_yaml::Value =
                    serde_yaml::from_str(&content).map_err(|e| format!("YAML parse error: {e}"))?;
                let entries = flatten_yaml_value(&yaml_value);
                return Ok(I18nAsset { entries });
            }
            #[cfg(feature = "po")]
            "po" => {
                let entries = parse_po(&content);
                return Ok(I18nAsset { entries });
            }
            _ => {}
        }

        Err(format!("Unsupported extension: '{extension}'").into())
    }

    fn extensions(&self) -> &[&str] {
        &[
            #[cfg(feature = "yaml")]
            "yaml",
            #[cfg(feature = "yaml")]
            "yml",
            #[cfg(feature = "po")]
            "po",
        ]
    }
}

/// Recursively flatten a serde_yaml::Value into dot-notation key-value pairs.
#[cfg(feature = "yaml")]
pub fn flatten_yaml_value(value: &serde_yaml::Value) -> HashMap<String, String> {
    let mut map = HashMap::new();
    flatten_recursive(value, String::new(), &mut map);
    map
}

#[cfg(feature = "yaml")]
fn flatten_recursive(
    value: &serde_yaml::Value,
    prefix: String,
    map: &mut HashMap<String, String>,
) {
    match value {
        serde_yaml::Value::String(s) => {
            map.insert(prefix, s.clone());
        }
        serde_yaml::Value::Mapping(m) => {
            for (key, val) in m {
                let key_str = match key {
                    serde_yaml::Value::String(s) => s.clone(),
                    serde_yaml::Value::Number(n) => n.to_string(),
                    _ => format!("{key:?}"),
                };
                let new_prefix = if prefix.is_empty() {
                    key_str
                } else {
                    format!("{prefix}.{key_str}")
                };
                flatten_recursive(val, new_prefix, map);
            }
        }
        other => {
            map.insert(prefix, format!("{other:?}"));
        }
    }
}

/// Minimal .po file parser. Handles msgid/msgstr and msgid_plural/msgstr[n].
#[cfg(feature = "po")]
fn parse_po(content: &str) -> HashMap<String, String> {
    let mut entries = HashMap::new();
    let mut current_msgid = String::new();
    let mut in_plural = false;

    for line in content.lines() {
        let line = line.trim();

        if line.starts_with("msgid_plural") {
            in_plural = true;
        } else if line.starts_with("msgid") && !line.starts_with("msgid_plural") {
            current_msgid = extract_po_string(line).unwrap_or_default();
            in_plural = false;
        } else if line.starts_with("msgstr[") {
            if let Some((idx, val)) = parse_msgstr_indexed(line) {
                let plural_key = format!("{current_msgid}.{idx}");
                entries.insert(plural_key, val);
            }
        } else if line.starts_with("msgstr") && !in_plural {
            if let Some(val) = extract_po_string(line) {
                entries.insert(current_msgid.clone(), val);
            }
        }
    }

    entries
}

#[cfg(feature = "po")]
fn extract_po_string(line: &str) -> Option<String> {
    let start = line.find('"')? + 1;
    let end = line.rfind('"')?;
    if start > end {
        return None;
    }
    Some(line[start..end].to_string())
}

#[cfg(feature = "po")]
fn parse_msgstr_indexed(line: &str) -> Option<(usize, String)> {
    let bracket_start = line.find('[')?;
    let bracket_end = line.find(']')?;
    let idx: usize = line[bracket_start + 1..bracket_end].parse().ok()?;
    let val = extract_po_string(line)?;
    Some((idx, val))
}

#[cfg(all(test, feature = "yaml"))]
mod tests {
    use super::*;

    #[test]
    fn test_flatten_simple() {
        let yaml = serde_yaml::from_str(
            r#"
game.title: "My Game"
menu.quit: "Quit"
"#,
        )
        .unwrap();
        let map = flatten_yaml_value(&yaml);
        assert_eq!(map.get("game.title").unwrap(), "My Game");
        assert_eq!(map.get("menu.quit").unwrap(), "Quit");
    }

    #[test]
    fn test_flatten_nested() {
        let yaml = serde_yaml::from_str(
            r#"
game:
  title: "My Game"
  version: "1.0"
"#,
        )
        .unwrap();
        let map = flatten_yaml_value(&yaml);
        assert_eq!(map.get("game.title").unwrap(), "My Game");
        assert_eq!(map.get("game.version").unwrap(), "1.0");
    }

    #[test]
    fn test_flatten_plural() {
        let yaml = serde_yaml::from_str(
            r#"
items:
  zero: "No items"
  one: "One item"
  other: "{count} items"
"#,
        )
        .unwrap();
        let map = flatten_yaml_value(&yaml);
        assert_eq!(map.get("items.zero").unwrap(), "No items");
        assert_eq!(map.get("items.other").unwrap(), "{count} items");
    }
}
