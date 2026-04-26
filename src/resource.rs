use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

use bevy::asset::{Assets, Handle};
use bevy::prelude::{Font, Resource};

use crate::asset::I18nAsset;
use crate::interpolate::interpolate;

/// Configuration for missing key warnings.
#[derive(Clone, Debug)]
pub struct MissingKeyConfig {
    /// Whether to warn on missing keys (default: true in debug, false in release).
    pub warn_on_missing: bool,
}

impl Default for MissingKeyConfig {
    fn default() -> Self {
        Self {
            #[cfg(debug_assertions)]
            warn_on_missing: true,
            #[cfg(not(debug_assertions))]
            warn_on_missing: false,
        }
    }
}

/// Central i18n resource — manages current locale and registered locale handles.
#[derive(Resource, Default)]
pub struct I18n {
    current_locale: String,
    fallback_locale: Option<String>,
    locale_map: HashMap<String, Handle<I18nAsset>>,
    locale_changed: bool,
    missing_key_config: MissingKeyConfig,
    /// Count of missing key lookups since last reset.
    missing_key_count: AtomicU64,
    /// Cache of (key, vars_hash) -> translated string. Cleared on locale change.
    translation_cache: Mutex<HashMap<(String, u64), String>>,
    /// Per-locale font handles. Used for automatic font fallback.
    locale_fonts: HashMap<String, Handle<Font>>,
}

impl I18n {
    /// Register a locale with its asset handle.
    pub fn add_locale(&mut self, locale: &str, handle: Handle<I18nAsset>) {
        self.locale_map.insert(locale.to_string(), handle);
    }

    /// Clear the translation cache (used during hot reload).
    pub fn clear_translation_cache(&mut self) {
        self.translation_cache.lock().unwrap().clear();
    }

    /// Set the current locale.
    pub fn set_locale(&mut self, locale: &str) {
        if self.current_locale != locale {
            self.current_locale = locale.to_string();
            self.translation_cache.lock().unwrap().clear();
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

    /// Set the font handle for a specific locale.
    pub fn set_locale_font(&mut self, locale: &str, handle: Handle<Font>) {
        self.locale_fonts.insert(locale.to_string(), handle);
    }

    /// Get the font handle for the current locale, if set.
    pub fn current_locale_font(&self) -> Option<&Handle<Font>> {
        self.locale_fonts.get(&self.current_locale)
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

        // Check cache
        let cache_key = hash_cache_key(&template_key, &resolved_vars);
        {
            let cache = self.translation_cache.lock().unwrap();
            if let Some(cached) = cache.get(&cache_key) {
                return cached.clone();
            }
        }

        match asset.get(&template_key) {
            Some(template) => {
                let resolved = interpolate(template, &resolved_refs).into_owned();
                self.translation_cache.lock().unwrap().insert(cache_key, resolved.clone());
                resolved
            }
            None => {
                let locale = self.current_locale.clone();
                match self.try_fallback(&template_key, &resolved_refs, locales) {
                    Some(result) => {
                        self.translation_cache.lock().unwrap().insert(cache_key.clone(), result.clone());
                        result
                    }
                    None => {
                        // Both current locale and fallback failed
                        self.warn_missing_key(key, &locale);
                        key.to_string()
                    }
                }
            }
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

    /// Configure missing key warning behavior.
    pub fn set_missing_key_config(&mut self, config: MissingKeyConfig) {
        self.missing_key_config = config;
    }

    /// Get the current missing key config.
    pub fn missing_key_config(&self) -> &MissingKeyConfig {
        &self.missing_key_config
    }

    /// Warn about a missing key and increment the counter.
    fn warn_missing_key(&self, key: &str, locale: &str) {
        self.missing_key_count.fetch_add(1, Ordering::Relaxed);
        if self.missing_key_config.warn_on_missing {
            eprintln!("[i18n] Missing key '{key}' in locale '{locale}'");
        }
    }

    /// Reset the missing key counter.
    pub fn reset_missing_key_count(&self) -> u64 {
        self.missing_key_count.swap(0, Ordering::Relaxed)
    }
}

/// Hash a (template_key, vars) pair for cache lookup.
fn hash_cache_key(key: &str, vars: &[(String, String)]) -> (String, u64) {
    let mut hasher = DefaultHasher::new();
    for (k, v) in vars {
        k.hash(&mut hasher);
        v.hash(&mut hasher);
    }
    (key.to_string(), hasher.finish())
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

    #[test]
    fn test_cache_returns_same_value() {
        let (assets, handle) = setup_i18n();

        let mut i18n = I18n::default();
        i18n.add_locale("en", handle);
        i18n.set_locale("en");

        // First lookup
        let text1 = i18n.get("greeting", &[], &assets);
        assert_eq!(text1, "Hello!");

        // Cache should return same value on second lookup
        let text2 = i18n.get("greeting", &[], &assets);
        assert_eq!(text2, "Hello!");

        // Verify cache is populated (non-empty)
        assert!(!i18n.translation_cache.lock().unwrap().is_empty());
    }

    #[test]
    fn test_cache_cleared_on_locale_change() {
        let (assets, handle) = setup_i18n();

        let mut i18n = I18n::default();
        i18n.add_locale("en", handle.clone());
        i18n.add_locale("zh", handle);
        i18n.set_locale("en");

        // Lookup to populate cache
        let _ = i18n.get("greeting", &[], &assets);
        assert!(!i18n.translation_cache.lock().unwrap().is_empty());

        // Switch locale — cache should be cleared
        i18n.set_locale("zh");
        assert!(i18n.translation_cache.lock().unwrap().is_empty());
    }
}
