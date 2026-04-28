use bevy::app::{App, Plugin, Update};
use bevy::asset::AssetApp;
use bevy::ui::prelude::Text;

use crate::asset::{I18nAsset, I18nLoader};
use crate::resource::I18n;
use crate::systems::{hot_reload_system, resolve_dynamic_vars, resolve_translations, update_fonts, update_localizable};

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
            .add_systems(Update, (
                resolve_translations,
                resolve_dynamic_vars,
                hot_reload_system,
                update_fonts,
                update_localizable::<Text>,
            ));
    }
}
