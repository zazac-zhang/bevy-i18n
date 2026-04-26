use bevy::asset::{AssetEvent, Assets};
use bevy::ecs::message::MessageReader;
use bevy::prelude::*;
use bevy::ui::prelude::Text;

use crate::asset::I18nAsset;
use crate::component::T;
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

/// Updates Text content for dirty T components.
pub fn update_text_system(
    i18n: Res<I18n>,
    locales: Res<Assets<I18nAsset>>,
    mut query: Query<(&mut T, &mut Text)>,
) {
    for (mut t, mut text) in &mut query {
        if !t.dirty {
            continue;
        }

        let vars: Vec<(&str, &str)> =
            t.vars.iter().map(|(k, v): &(String, String)| (k.as_str(), v.as_str())).collect::<Vec<_>>();

        let translated = i18n.get_plural(&t.key, t.count, &vars, &locales);

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
