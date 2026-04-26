use std::collections::HashMap;

use bevy::asset::{Assets, Handle};
use bevy::prelude::Resource;

use crate::asset::I18nAsset;
use crate::interpolate::interpolate;

/// Central i18n resource — manages current locale and registered locale handles.
#[derive(Resource, Default)]
pub struct I18n {
    current_locale: String,
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
        let Some(handle) = self.locale_map.get(&self.current_locale) else {
            return key.to_string();
        };

        let Some(asset) = locales.get(handle.id()) else {
            return key.to_string();
        };

        let Some(template) = asset.get(key) else {
            return key.to_string();
        };

        interpolate(template, vars).into_owned()
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
}
