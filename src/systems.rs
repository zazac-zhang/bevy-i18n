use bevy::asset::{AssetEvent, Assets};
use bevy::ecs::message::MessageReader;
use bevy::prelude::*;
use bevy::text::TextFont;
use bevy::ui::prelude::Text;

use crate::asset::I18nAsset;
use crate::component::{T, TVar};
use crate::resource::I18n;

/// Resolves translation keys for all T components.
/// Marks dirty T components that need text updates.
pub fn resolve_translations(
    mut i18n: ResMut<I18n>,
    mut query: Query<&mut T>,
) {
    let locale_changed = i18n.update_from();

    if locale_changed {
        for mut t in &mut query {
            t.mark_dirty();
        }
    }
}

/// Checks dynamic variable references on T components.
/// Marks T dirty when referenced TVar values have changed.
pub fn resolve_dynamic_vars(
    mut query: Query<&mut T>,
    tvar_query: Query<&TVar>,
) {
    for mut t in &mut query {
        if t.dynamic_vars.is_empty() {
            continue;
        }

        // Collect current dynamic var values
        let current_values: Vec<(String, String)> = t
            .dynamic_vars
            .iter()
            .filter_map(|(name, entity)| tvar_query.get(*entity).ok().map(|tv| (name.clone(), tv.value.clone())))
            .collect();

        if current_values.is_empty() {
            continue;
        }

        // Check if any value changed since last frame
        let last_values = t
            .vars
            .iter()
            .filter(|(k, _)| t.dynamic_vars.iter().any(|(dn, _)| dn == k))
            .cloned()
            .collect::<Vec<_>>();

        if current_values != last_values {
            t.mark_dirty();
        }
    }
}

/// Updates Text content for dirty T components.
pub fn update_text_system(
    i18n: Res<I18n>,
    locales: Res<Assets<I18nAsset>>,
    mut query: Query<(&mut T, &mut Text)>,
    tvar_query: Query<&TVar>,
    mut text_fonts: Query<&mut TextFont, With<T>>,
) {
    // Update fonts for all T components if locale font is set
    if let Some(font_handle) = i18n.current_locale_font() {
        for mut text_font in &mut text_fonts {
            text_font.font = font_handle.clone();
        }
    }

    for (mut t, mut text) in &mut query {
        if !t.dirty {
            continue;
        }

        // Build combined vars: static + dynamic
        let mut all_vars: Vec<(String, String)> = t.vars.clone();
        for (name, entity) in &t.dynamic_vars {
            if let Ok(tvar) = tvar_query.get(*entity) {
                all_vars.push((name.clone(), tvar.value.clone()));
            }
        }

        let vars: Vec<(&str, &str)> =
            all_vars.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect::<Vec<_>>();

        let translated = i18n.get_plural(&t.key, t.context.as_deref(), t.count, &vars, &locales);

        // Update the text content and clear the dirty flag
        text.0 = translated;
        t.dirty = false;
    }
}

/// Listens for I18nAsset changes and marks all T components dirty on reload.
pub fn hot_reload_system(
    mut events: MessageReader<AssetEvent<I18nAsset>>,
    mut i18n: ResMut<I18n>,
    mut query: Query<&mut T>,
) {
    for event in events.read() {
        if matches!(
            event,
            AssetEvent::LoadedWithDependencies { .. } | AssetEvent::Modified { .. }
        ) {
            // Clear translation cache and mark all T components dirty
            i18n.clear_translation_cache();
            for mut t in &mut query {
                t.mark_dirty();
            }
        }
    }
}
