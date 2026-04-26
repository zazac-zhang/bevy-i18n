//! Minimal example: load locales and display translated text.
//!
//! Run with: `cargo run --example basic`

use bevy::prelude::*;
use bevy_i18n::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(I18nPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    asset_server: Res<AssetServer>,
    mut i18n: ResMut<I18n>,
    mut commands: Commands,
) {
    // Load locale files from examples/locales/
    let en = asset_server.load("locales/en.yaml");
    let zh = asset_server.load("locales/zh.yaml");

    i18n.add_locale("en", en);
    i18n.add_locale("zh", zh);

    // Set the active locale
    i18n.set_locale("en");

    // Spawn translated text — the `T` component resolves the key automatically
    commands.spawn((
        Text::new(""),
        T::new("game.title"),
    ));

    // With variable substitutions
    commands.spawn((
        Text::new(""),
        T::with_vars("player.greeting", &[("name", "Player")]),
    ));

    // Plural forms based on count
    commands.spawn((
        Text::new(""),
        T::plural("player.inventory", 0),
    ));
}
