use bevy::asset::{AssetEvent, Assets};
use bevy::ecs::component::Mutable;
use bevy::ecs::message::MessageReader;
use bevy::prelude::*;
use bevy::text::TextFont;

use crate::asset::I18nAsset;
use crate::component::{I18nMarker, Localizable, TVar};
use crate::resource::I18n;

/// Marks all I18nMarker components dirty on locale change.
pub fn resolve_translations(
    mut i18n: ResMut<I18n>,
    mut query: Query<&mut I18nMarker>,
) {
    let locale_changed = i18n.update_from();
    if locale_changed {
        for mut m in &mut query {
            m.mark_dirty();
        }
    }
}

/// Checks dynamic variable references on I18nMarker components.
pub fn resolve_dynamic_vars(
    mut query: Query<&mut I18nMarker>,
    tvar_query: Query<&TVar>,
) {
    for mut m in &mut query {
        if m.dynamic_vars.is_empty() {
            continue;
        }
        let current_values: Vec<(String, String)> = m
            .dynamic_vars
            .iter()
            .filter_map(|(name, entity)| {
                tvar_query.get(*entity).ok().map(|tv| (name.clone(), tv.value.clone()))
            })
            .collect();
        if current_values.is_empty() {
            continue;
        }
        let last_values = m
            .vars
            .iter()
            .filter(|(k, _)| m.dynamic_vars.iter().any(|(dn, _)| dn == k))
            .cloned()
            .collect::<Vec<_>>();
        if current_values != last_values {
            m.mark_dirty();
        }
    }
}

/// Generic translation update for any `Localizable` component.
///
/// If `I18nMarker.key` is set, only the matching field is translated (Text / single-field path).
/// If `I18nMarker.key` is `None`, all `T::translations()` fields are translated (multi-field path).
///
/// Register with `app.add_systems(Update, update_localizable::<MyComponent>)`.
pub fn update_localizable<T: Localizable + Component<Mutability = Mutable>>(
    i18n: Res<I18n>,
    locales: Res<Assets<I18nAsset>>,
    mut query: Query<(&mut I18nMarker, &mut T)>,
    tvar_query: Query<&TVar>,
) {
    for (mut m, mut component) in query.iter_mut() {
        if !m.dirty {
            continue;
        }

        // Collect vars from marker + dynamic
        let mut all_vars: Vec<(String, String)> = m.vars.clone();
        for (name, entity) in &m.dynamic_vars {
            if let Ok(tvar) = tvar_query.get(*entity) {
                all_vars.push((name.clone(), tvar.value.clone()));
            }
        }
        let vars: Vec<(&str, &str)> =
            all_vars.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();

        if let Some(ref key) = m.key {
            // Key-based: translate this specific key
            let translated = i18n.get_plural(key, m.context.as_deref(), m.count, &vars, &locales);
            let translations = T::translations();
            if translations.len() == 1 && translations[0].1 == "_text_" {
                // Single-field text: always update (key matches the Text component, not a field)
                component.set_field(translations[0].0, &translated);
            } else {
                // Multi-field component: match key to find the right field
                for (field_name, field_key) in translations {
                    if field_key == key {
                        component.set_field(field_name, &translated);
                    }
                }
            }
        } else {
            // Marker-only: translate all fields via translations()
            for (field_name, field_key) in T::translations() {
                let translated = i18n.get_plural(field_key, None, None, &vars, &locales);
                component.set_field(field_name, &translated);
            }
        }

        m.dirty = false;
    }
}

/// Updates font handles for entities with I18nMarker + TextFont.
pub fn update_fonts(
    i18n: Res<I18n>,
    mut text_fonts: Query<&mut TextFont, With<I18nMarker>>,
) {
    if let Some(font_handle) = i18n.current_locale_font() {
        for mut text_font in &mut text_fonts {
            text_font.font = font_handle.clone();
        }
    }
}

/// Listens for I18nAsset changes and marks all I18nMarker components dirty on reload.
pub fn hot_reload_system(
    mut events: MessageReader<AssetEvent<I18nAsset>>,
    mut i18n: ResMut<I18n>,
    mut query: Query<&mut I18nMarker>,
) {
    for event in events.read() {
        if matches!(
            event,
            AssetEvent::LoadedWithDependencies { .. } | AssetEvent::Modified { .. }
        ) {
            i18n.clear_translation_cache();
            for mut m in &mut query {
                m.mark_dirty();
            }
        }
    }
}
