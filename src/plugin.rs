use bevy::app::{App, Plugin, Update};
use bevy::asset::AssetApp;

use crate::asset::{I18nAsset, I18nLoader};
use crate::resource::I18n;
use crate::systems::{hot_reload_system, resolve_translations, update_text_system};

/// Plugin that registers all i18n types, assets, loaders, and systems.
///
/// # Usage
/// ```ignore
/// App::new()
///     .add_plugins(DefaultPlugins)
///     .add_plugins(I18nPlugin)
/// ```
pub struct I18nPlugin;

impl Plugin for I18nPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<I18nAsset>()
            .init_asset_loader::<I18nLoader>()
            .init_resource::<I18n>()
            .add_systems(Update, (resolve_translations, update_text_system, hot_reload_system));
    }
}
