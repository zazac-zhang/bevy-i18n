use std::collections::HashMap;

use bevy::asset::{Assets, Handle};
use bevy::prelude::Resource;

use crate::asset::I18nAsset;
use crate::interpolate::interpolate;

/// Central i18n resource — manages current locale and registered locale handles.
#[derive(Resource, Default)]
pub struct I18n {
    current_locale: String,
    fallback_locale: Option<String>,
    locale_map: HashMap<String, Handle<I18nAsset>>,
    locale_changed: bool,
}

impl I18n {
    /// Register a locale with its asset handle.
    pub fn add_locale(&mut self, locale: &str, handle: Handle<I18nAsset>) {
        self.locale_map.insert(locale.to_string(), handle);
    }

    /// Set the current locale.
    pub fn set_locale(&mut self, locale: &str) {
        if self.current_locale != locale {
            self.current_locale = locale.to_string();
            self.locale_changed = true;
        }
    }

    /// Set the fallback locale. Used when a key is missing in the current locale.
    pub fn set_fallback_locale(&mut self, locale: impl Into<String>) {
        self.fallback_locale = Some(locale.into());
    }

    /// Get current locale identifier.
    pub fn current_locale(&self) -> &str {
        &self.current_locale
    }

    /// Check if locale has changed since last call.
    /// Call once per frame in the text update system.
    pub fn update_from(&mut self) -> bool {
        let was_changed = self.locale_changed;
        self.locale_changed = false;
        was_changed
    }

    /// Look up a translation key with optional variable interpolation.
    /// Returns the translated string, or the key itself if not found.
    pub fn get(&self, key: &str, vars: &[(&str, &str)], locales: &Assets<I18nAsset>) -> String {
        self.get_plural(key, None, vars, locales)
    }

    /// Look up a translation key with plural form selection and variable interpolation.
    ///
    /// If `count` is Some, selects the appropriate plural form:
    /// - `0` → `{key}.zero` (falls back to `{key}.other`)
    /// - `1` → `{key}.one`
    /// - `2+` → `{key}.other`
    ///
    /// Returns the translated string, or the key itself if not found.
    pub fn get_plural(
        &self,
        key: &str,
        count: Option<u64>,
        vars: &[(&str, &str)],
        locales: &Assets<I18nAsset>,
    ) -> String {
        let Some(handle) = self.locale_map.get(&self.current_locale) else {
            return key.to_string();
        };

        let Some(asset) = locales.get(handle.id()) else {
            return key.to_string();
        };

        // Resolve the actual key based on count
        let template_key = match count {
            None => key.to_string(),
            Some(0) => asset
                .get(&format!("{key}.zero"))
                .map(|_| format!("{key}.zero"))
                .or_else(|| asset.get(&format!("{key}.other")).map(|_| format!("{key}.other")))
                .unwrap_or_else(|| key.to_string()),
            Some(1) => asset
                .get(&format!("{key}.one"))
                .map(|_| format!("{key}.one"))
                .unwrap_or_else(|| key.to_string()),
            Some(_) => asset
                .get(&format!("{key}.other"))
                .map(|_| format!("{key}.other"))
                .unwrap_or_else(|| key.to_string()),
        };

        // Inject count into vars if not already present
        let resolved_vars: Vec<(String, String)> = if let Some(c) = count {
            let mut v: Vec<(String, String)> = vars
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect();
            if !v.iter().any(|(k, _)| k == "count") {
                v.push(("count".to_string(), c.to_string()));
            }
            v
        } else {
            vars.iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect()
        };

        let resolved_refs: Vec<(&str, &str)> = resolved_vars
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();

        match asset.get(&template_key) {
            Some(template) => interpolate(template, &resolved_refs).into_owned(),
            None => self
                .try_fallback(&template_key, &resolved_refs, locales)
                .unwrap_or_else(|| key.to_string()),
        }
    }

    fn try_fallback(
        &self,
        key: &str,
        vars: &[(&str, &str)],
        locales: &Assets<I18nAsset>,
    ) -> Option<String> {
        let fallback = self.fallback_locale.as_ref()?;
        let handle = self.locale_map.get(fallback)?;
        let fallback_asset = locales.get(handle.id())?;
        let template = fallback_asset.get(key)?;
        Some(interpolate(template, vars).into_owned())
    }

    /// Check if a locale's asset is loaded.
    pub fn is_locale_loaded(&self, locale: &str, locales: &Assets<I18nAsset>) -> bool {
        self.locale_map
            .get(locale)
            .and_then(|h| locales.get(h.id()))
            .is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::asset::Assets;

    fn setup_i18n() -> (Assets<I18nAsset>, Handle<I18nAsset>) {
        let mut assets = Assets::<I18nAsset>::default();

        let mut entries = std::collections::HashMap::new();
        entries.insert("greeting".to_string(), "Hello!".to_string());
        entries.insert("farewell".to_string(), "Goodbye!".to_string());
        let asset = I18nAsset::new(entries);

        let handle = assets.add(asset);

        (assets, handle)
    }

    #[test]
    fn test_add_locale_and_lookup() {
        let (assets, handle) = setup_i18n();

        let mut i18n = I18n::default();
        i18n.add_locale("en", handle.clone());
        i18n.set_locale("en");

        let text = i18n.get("greeting", &[], &assets);
        assert_eq!(text, "Hello!");
    }

    #[test]
    fn test_locale_change_detection() {
        let (_assets, handle) = setup_i18n();

        let mut i18n = I18n::default();
        i18n.add_locale("en", handle);
        i18n.set_locale("en");

        // First call: locale was just set, so change detected
        assert!(i18n.update_from());
        // Second call: no change since last call
        assert!(!i18n.update_from());
    }

    #[test]
    fn test_missing_key_returns_key() {
        let (assets, handle) = setup_i18n();

        let mut i18n = I18n::default();
        i18n.add_locale("en", handle);
        i18n.set_locale("en");

        // Missing key returns the key itself, not None
        let text = i18n.get("nonexistent", &[], &assets);
        assert_eq!(text, "nonexistent");
    }

    #[test]
    fn test_interpolation_through_get() {
        let mut assets = Assets::<I18nAsset>::default();

        let mut entries = std::collections::HashMap::new();
        entries.insert("greet".to_string(), "Hello, {name}!".to_string());
        let asset = I18nAsset::new(entries);
        let handle = assets.add(asset);

        let mut i18n = I18n::default();
        i18n.add_locale("en", handle);
        i18n.set_locale("en");

        let text = i18n.get("greet", &[("name", "World")], &assets);
        assert_eq!(text, "Hello, World!");
    }

    #[test]
    fn test_plural_selection() {
        let mut assets = Assets::<I18nAsset>::default();

        let mut entries = std::collections::HashMap::new();
        entries.insert("items.zero".to_string(), "No items".to_string());
        entries.insert("items.one".to_string(), "{count} item".to_string());
        entries.insert("items.other".to_string(), "{count} items".to_string());
        let asset = I18nAsset::new(entries);
        let handle = assets.add(asset);

        let mut i18n = I18n::default();
        i18n.add_locale("en", handle);
        i18n.set_locale("en");

        assert_eq!(i18n.get_plural("items", Some(0), &[], &assets), "No items");
        assert_eq!(i18n.get_plural("items", Some(1), &[], &assets), "1 item");
        assert_eq!(i18n.get_plural("items", Some(5), &[], &assets), "5 items");
        assert_eq!(i18n.get_plural("items", Some(0), &[], &assets), "No items");
    }

    #[test]
    fn test_fallback_locale() {
        let mut assets = Assets::<I18nAsset>::default();

        // English has all keys
        let mut en_entries = std::collections::HashMap::new();
        en_entries.insert("greeting".to_string(), "Hello!".to_string());
        en_entries.insert("farewell".to_string(), "Goodbye!".to_string());
        let en_handle = assets.add(I18nAsset::new(en_entries));

        // Chinese only has greeting, missing farewell
        let mut zh_entries = std::collections::HashMap::new();
        zh_entries.insert("greeting".to_string(), "你好！".to_string());
        let zh_handle = assets.add(I18nAsset::new(zh_entries));

        let mut i18n = I18n::default();
        i18n.add_locale("en_US", en_handle);
        i18n.add_locale("zh_CN", zh_handle);
        i18n.set_locale("zh_CN");
        i18n.set_fallback_locale("en_US");

        // Key exists in zh_CN - should use zh translation
        assert_eq!(i18n.get("greeting", &[], &assets), "你好！");

        // Key missing in zh_CN - should fallback to en_US
        assert_eq!(i18n.get("farewell", &[], &assets), "Goodbye!");
    }
}
