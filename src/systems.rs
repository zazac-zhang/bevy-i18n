use bevy::asset::{AssetEvent, Assets};
use bevy::ecs::component::Mutable;
use bevy::ecs::message::MessageReader;
use bevy::prelude::*;
use bevy::text::TextFont;
use bevy::ui::prelude::Text;

use crate::asset::I18nAsset;
use crate::component::{I18nText, Localizable, TVar};
use crate::resource::I18n;

/// Resolves translation keys for all I18nText components.
pub fn resolve_translations(
    mut i18n: ResMut<I18n>,
    mut query: Query<&mut I18nText>,
) {
    let locale_changed = i18n.update_from();
    if locale_changed {
        for mut t in &mut query {
            t.mark_dirty();
        }
    }
}

/// Checks dynamic variable references on I18nText components.
pub fn resolve_dynamic_vars(
    mut query: Query<&mut I18nText>,
    tvar_query: Query<&TVar>,
) {
    for mut t in &mut query {
        if t.dynamic_vars.is_empty() {
            continue;
        }
        let current_values: Vec<(String, String)> = t
            .dynamic_vars
            .iter()
            .filter_map(|(name, entity)| {
                tvar_query.get(*entity).ok().map(|tv| (name.clone(), tv.value.clone()))
            })
            .collect();
        if current_values.is_empty() {
            continue;
        }
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

/// Updates Text content for dirty I18nText components.
pub fn update_text_system(
    i18n: Res<I18n>,
    locales: Res<Assets<I18nAsset>>,
    mut query: Query<(&mut I18nText, &mut Text)>,
    tvar_query: Query<&TVar>,
    mut text_fonts: Query<&mut TextFont, With<I18nText>>,
) {
    if let Some(font_handle) = i18n.current_locale_font() {
        for mut text_font in &mut text_fonts {
            text_font.font = font_handle.clone();
        }
    }
    for (mut t, mut text) in &mut query {
        if !t.dirty {
            continue;
        }
        let mut all_vars: Vec<(String, String)> = t.vars.clone();
        for (name, entity) in &t.dynamic_vars {
            if let Ok(tvar) = tvar_query.get(*entity) {
                all_vars.push((name.clone(), tvar.value.clone()));
            }
        }
        let vars: Vec<(&str, &str)> =
            all_vars.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect::<Vec<_>>();
        let translated = i18n.get_plural(&t.key, t.context.as_deref(), t.count, &vars, &locales);
        text.0 = translated;
        t.dirty = false;
    }
}

/// Listens for I18nAsset changes and marks all I18nText components dirty on reload.
pub fn hot_reload_system(
    mut events: MessageReader<AssetEvent<I18nAsset>>,
    mut i18n: ResMut<I18n>,
    mut query: Query<&mut I18nText>,
) {
    for event in events.read() {
        if matches!(
            event,
            AssetEvent::LoadedWithDependencies { .. } | AssetEvent::Modified { .. }
        ) {
            i18n.clear_translation_cache();
            for mut t in &mut query {
                t.mark_dirty();
            }
        }
    }
}

/// Generic update system for any `Localizable` component paired with `I18nText`.
///
/// Register with `app.add_systems(Update, update_localizable::<MyComponent>)`.
pub fn update_localizable<T: Localizable + Component<Mutability = Mutable>>(
    i18n: Res<I18n>,
    locales: Res<Assets<I18nAsset>>,
    mut query: Query<(&mut I18nText, &mut T)>,
) {
    for (mut t, mut component) in query.iter_mut() {
        if !t.dirty {
            continue;
        }
        let vars: Vec<(&str, &str)> = t
            .vars
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        let translation = i18n.get_plural(
            &t.key,
            t.context.as_deref(),
            t.count,
            &vars,
            &locales,
        );
        for (field_name, _key) in T::translations() {
            component.set_field(field_name, &translation);
        }
        t.dirty = false;
    }
}

/// Alias for update_localizable.
pub fn update_i18n<T: Localizable + Component<Mutability = Mutable>>(
    i18n: Res<I18n>,
    locales: Res<Assets<I18nAsset>>,
    query: Query<(&mut I18nText, &mut T)>,
) {
    update_localizable::<T>(i18n, locales, query)
}
