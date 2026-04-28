//! Custom component demo using the `#[derive(I18n)]` macro.
//!
//! Shows how to use the derive macro to translate custom components
//! instead of Bevy's standard Text component.
//!
//! Run with: `cargo run --example custom_component --features derive`

use bevy::prelude::*;
use bevy_i18n::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(I18nPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, (
            handle_input,
            update_localizable::<DialogBox>,
            update_localizable::<HUD>,
        ))
        .run();
}

/// Custom dialog box with multiple translatable fields
#[derive(I18n, Component)]
struct DialogBox {
    #[i18n(key = "dialog.title")]
    title: String,
    #[i18n(key = "dialog.body")]
    body: String,
    // Non-String fields are auto-ignored
    bg_color: Color,
}

/// HUD component using namespace prefix
#[derive(I18n, Component)]
#[i18n(namespace = "hud")]
struct HUD {
    #[i18n(key = "score")]
    score: String,
    #[i18n(key = "level")]
    level: String,
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
    i18n.set_fallback_locale("en");

    commands.spawn(Camera2d);

    // Spawn custom dialog box — I18nMarker::marker() triggers translation
    // of all fields defined in DialogBox::translations()
    commands.spawn((
        I18nMarker::marker(),
        DialogBox {
            title: String::new(),
            body: String::new(),
            bg_color: Color::srgb(0.1, 0.1, 0.1),
        },
    ));

    // Spawn custom HUD — same pattern: empty marker, all fields translated
    commands.spawn((
        I18nMarker::marker(),
        HUD {
            score: String::new(),
            level: String::new(),
        },
    ));
}

fn handle_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut i18n: ResMut<I18n>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        let current = i18n.current_locale().to_string();
        i18n.set_locale(if current == "en" { "zh" } else { "en" });
    }
}
