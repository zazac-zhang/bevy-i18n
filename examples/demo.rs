//! Comprehensive example showing all `bevy_i18n` features in one file.
//!
//! Run with: `cargo run --example demo`
//! Run with derive: `cargo run --example demo --features derive`
//!
//! Controls:
//! - Space — toggle EN/ZH locale
//! - ↑/↓ — change item count (plural demo)
//! - +/- — change score (dynamic variable demo)

use bevy::prelude::*;
use bevy_i18n::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(I18nPlugin)
        .init_resource::<AppState>()
        .add_systems(Startup, setup)
        .add_systems(Update, (
            handle_input,
            update_localizable::<DialogBox>,
            update_localizable::<ManualLabel>,
        ))
        .run();
}

// ── Custom components ──────────────────────────────────────────────

/// Via `#[derive(I18n)]` macro (requires `derive` feature).
#[cfg(feature = "derive")]
mod derive_component {
    use bevy::prelude::*;
    use bevy_i18n::prelude::*;

    #[derive(I18n, Component)]
    #[i18n(namespace = "dialog")]
    pub struct DialogBox {
        #[i18n(key = "title")]
        pub title: String,
        #[i18n(key = "body")]
        pub body: String,
    }
}

#[cfg(feature = "derive")]
use derive_component::DialogBox;

/// Stub when derive feature is off — implements Localizable manually.
#[cfg(not(feature = "derive"))]
mod derive_component {
    use bevy::prelude::*;
    use bevy_i18n::prelude::*;

    #[derive(Component)]
    pub struct DialogBox {
        pub title: String,
        pub body: String,
    }

    impl Localizable for DialogBox {
        fn translations() -> &'static [(&'static str, &'static str)] {
            &[
                ("title", "dialog.title"),
                ("body", "dialog.body"),
            ]
        }
        fn set_field(&mut self, field: &str, value: &str) {
            match field {
                "title" => self.title = value.into(),
                "body" => self.body = value.into(),
                _ => {}
            }
        }
    }
}

#[cfg(not(feature = "derive"))]
use derive_component::DialogBox;

/// Manual `Localizable` impl — for third-party components you can't derive on.
#[derive(Component)]
struct ManualLabel {
    text: String,
}

impl Localizable for ManualLabel {
    fn translations() -> &'static [(&'static str, &'static str)] {
        &[("text", "menu.new_game")]
    }
    fn set_field(&mut self, _field: &str, value: &str) {
        self.text = value.into();
    }
}

// ── Marker components for querying ─────────────────────────────────

#[derive(Component)]
struct PluralLabel;

#[derive(Component)]
struct LocaleLabel;

// ── State ──────────────────────────────────────────────────────────

#[derive(Resource)]
struct AppState {
    score_entity: Entity,
    item_count: u64,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            score_entity: Entity::PLACEHOLDER,
            item_count: 5,
        }
    }
}

// ── Setup ──────────────────────────────────────────────────────────

fn setup(
    asset_server: Res<AssetServer>,
    mut i18n: ResMut<I18n>,
    mut state: ResMut<AppState>,
    mut commands: Commands,
) {
    commands.spawn(Camera2d);

    // Load locales
    let en = asset_server.load("locales/en.yaml");
    let zh = asset_server.load("locales/zh.yaml");
    i18n.add_locale("en", en);
    i18n.add_locale("zh", zh);
    i18n.set_locale("en");

    // 1. Simple key
    commands.spawn((Text::new(""), I18nMarker::new("game.title")));

    // 2. With variables
    commands.spawn((
        Text::new(""),
        I18nMarker::with_vars("player.greeting", &[("name", "Adventurer")]),
    ));

    // 3. Plural forms
    commands.spawn((
        Text::new(""),
        I18nMarker::plural("player.inventory", 5),
        PluralLabel,
    ));

    // 4. Namespace builder
    commands.spawn((
        Text::new(""),
        I18nMarker::ns("menu").key("quit"),
    ));

    // 5. Dynamic variables (TVar)
    let score_entity = commands.spawn(TVar::new("0")).id();
    commands.spawn((
        Text::new(""),
        I18nMarker::new("player.score").with_dynamic_var("score", score_entity),
    ));
    state.score_entity = score_entity;

    // 6. Custom component (derive or manual Localizable)
    commands.spawn((
        I18nMarker::marker(),
        DialogBox {
            title: String::new(),
            body: String::new(),
        },
    ));

    // 7. Manual Localizable for third-party component
    commands.spawn((
        I18nMarker::marker(),
        ManualLabel { text: String::new() },
    ));

    // Locale indicator
    commands.spawn((
        Text::new(""),
        LocaleLabel,
    ));

    // Control hints
    commands.spawn(Text::new(
        "Space: switch locale | ↑↓: count | +/-: score",
    ));
}

// ── Input handling ─────────────────────────────────────────────────

fn handle_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<AppState>,
    mut i18n: ResMut<I18n>,
    mut plural_query: Query<&mut I18nMarker, With<PluralLabel>>,
    mut score_query: Query<&mut TVar>,
    mut locale_query: Query<&mut Text, With<LocaleLabel>>,
) {
    // Toggle locale
    if keyboard.just_pressed(KeyCode::Space) {
        let next = if i18n.current_locale() == "en" { "zh" } else { "en" };
        i18n.set_locale(next);
    }

    // Change item count (plural)
    if keyboard.just_pressed(KeyCode::ArrowUp) {
        state.item_count += 1;
        for mut m in &mut plural_query {
            *m = I18nMarker::plural("player.inventory", state.item_count);
        }
    }
    if keyboard.just_pressed(KeyCode::ArrowDown) {
        state.item_count = state.item_count.saturating_sub(1);
        for mut m in &mut plural_query {
            *m = I18nMarker::plural("player.inventory", state.item_count);
        }
    }

    // Change score (dynamic var)
    if keyboard.just_pressed(KeyCode::Equal) {
        if let Ok(mut tvar) = score_query.get_mut(state.score_entity) {
            let current: u64 = tvar.value.parse().unwrap_or(0);
            tvar.value = (current + 100).to_string();
        }
    }
    if keyboard.just_pressed(KeyCode::Minus) {
        if let Ok(mut tvar) = score_query.get_mut(state.score_entity) {
            let current: u64 = tvar.value.parse().unwrap_or(0);
            tvar.value = current.saturating_sub(100).to_string();
        }
    }

    // Update locale indicator
    for mut text in &mut locale_query {
        let label = match i18n.current_locale() {
            "en" => "[EN] English",
            "zh" => "[ZH] 中文",
            other => other,
        };
        text.0 = label.to_string();
    }
}
