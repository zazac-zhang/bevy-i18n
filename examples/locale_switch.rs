//! Runtime locale switching example.
//!
//! Press <kbd>Space</kbd> to toggle between English and Chinese.
//! All `T` components update automatically.
//!
//! Run with: `cargo run --example locale_switch`

use bevy::prelude::*;
use bevy_i18n::prelude::*;

const LOCALES: [&str; 2] = ["en", "zh"];

#[derive(Resource, Default)]
struct LocaleIndex(usize);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(I18nPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, handle_input)
        .add_systems(Update, update_locale_label.after(handle_input))
        .run();
}

fn setup(
    asset_server: Res<AssetServer>,
    mut i18n: ResMut<I18n>,
    mut commands: Commands,
) {
    let en = asset_server.load("locales/en.yaml");
    let zh = asset_server.load("locales/zh.yaml");

    i18n.add_locale("en", en);
    i18n.add_locale("zh", zh);
    i18n.set_locale("en");

    // Track which locale we're currently on
    commands.insert_resource(LocaleIndex::default());

    // Title
    commands.spawn((
        Text::new(""),
        T::new("game.title"),
    ));

    // Subtitle with namespace builder
    commands.spawn((
        Text::new(""),
        T::ns("game").key("subtitle"),
    ));

    // Current language indicator
    commands.spawn((
        Text::new(""),
        LanguageLabel,
    ));

    // Prompt
    commands.spawn((
        Text::new("Press Space to switch language"),
    ));
}

/// Marker component for the language label entity
#[derive(Component)]
struct LanguageLabel;

fn handle_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut locale_idx: ResMut<LocaleIndex>,
    mut i18n: ResMut<I18n>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        locale_idx.0 = (locale_idx.0 + 1) % LOCALES.len();
        i18n.set_locale(LOCALES[locale_idx.0]);
    }
}

fn update_locale_label(
    i18n: Res<I18n>,
    mut query: Query<&mut Text, With<LanguageLabel>>,
) {
    let label = match i18n.current_locale() {
        "en" => "[English]",
        "zh" => "[中文]",
        other => other,
    };
    for mut text in &mut query {
        text.0 = label.to_string();
    }
}
