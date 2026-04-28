//! Advanced features demo: context disambiguation, dynamic variables,
//! number/currency formatting, fallback locales, and namespace builder.
//!
//! This example showcases every major feature of `bevy_i18n`.
//!
//! Controls:
//! - Arrow Up/Down — change item count (plural demo)
//! - Arrow Left/Right — switch locale
//! - + / - — change score (dynamic variable demo)
//!
//! Run with: `cargo run --example advanced_features`

use bevy::prelude::*;
use bevy_i18n::prelude::*;
use bevy_i18n::NumberFormat;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(I18nPlugin)
        .init_resource::<AppState>()
        .add_systems(Startup, setup)
        .add_systems(Update, handle_input)
        .run();
}

#[derive(Resource)]
struct AppState {
    /// Entity with TVar for the score
    score_entity: Entity,
    /// Current item count (for plural demo)
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

fn setup(
    asset_server: Res<AssetServer>,
    mut i18n: ResMut<I18n>,
    mut commands: Commands,
) {
    // ── Load locales ──────────────────────────────────────────────
    let en = asset_server.load("locales/en.yaml");
    let zh = asset_server.load("locales/zh.yaml");
    i18n.add_locale("en", en);
    i18n.add_locale("zh", zh);
    i18n.set_locale("en");
    i18n.set_fallback_locale("en");

    // ── Number / currency formatting ──────────────────────────────
    i18n.set_locale_number_format("en", NumberFormat {
        thousands_sep: ',',
        decimal_sep: '.',
        decimal_places: Some(2),
        currency_symbol: Some("$".to_string()),
    });
    i18n.set_locale_number_format("zh", NumberFormat {
        thousands_sep: ',',
        decimal_sep: '.',
        decimal_places: Some(2),
        currency_symbol: Some("¥".to_string()),
    });

    // ── Spawn dynamic variables ───────────────────────────────────
    let score_entity = commands.spawn(TVar::new("0")).id();

    // ── Spawn text with different T constructors ──────────────────

    // Simple key lookup
    commands.spawn((Text::new(""), I18nMarker::new("game.title")));

    // Static variable substitutions
    commands.spawn((Text::new(""), I18nMarker::with_vars("player.greeting", &[("name", "Adventurer")])));

    // Plural forms (count can be changed at runtime)
    commands.spawn((Text::new(""), I18nMarker::plural("player.inventory", 5), PluralLabel));

    // Context disambiguation — same key, different meanings
    commands.spawn((Text::new(""), I18nMarker::with_context("open", "menu")));
    commands.spawn((Text::new(""), I18nMarker::with_context("open", "dialog")));

    // Namespace builder — convenient for large projects
    commands.spawn((Text::new(""), I18nMarker::ns("menu").key("new_game")));
    commands.spawn((Text::new(""), I18nMarker::ns("menu").key("quit")));

    // Dynamic variable — references a TVar entity that updates at runtime
    commands.spawn((
        Text::new(""),
        I18nMarker::new("player.score").with_dynamic_var("score", score_entity),
    ));

    // Currency formatting via {amount::currency} specifier
    commands.spawn((
        Text::new(""),
        I18nMarker::ns("shop").with_vars("price", &[("amount", "1234.50")]),
    ));

    // Missing key — falls back to the key string itself (with debug warning)
    commands.spawn((Text::new(""), I18nMarker::new("this.key.does.not.exist")));

    // Store state
    commands.insert_resource(AppState {
        score_entity,
        item_count: 5,
    });

    // Control hints
    commands.spawn((
        Text::new("↑↓ change count | ←→ switch locale | +− change score"),
    ));
}

/// Marker for the plural entity (so we can update it)
#[derive(Component)]
struct PluralLabel;

fn handle_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<AppState>,
    mut i18n: ResMut<I18n>,
    mut plural_query: Query<&mut I18nMarker, With<PluralLabel>>,
    mut score_query: Query<&mut TVar>,
) {
    // Change item count → triggers plural re-selection
    if keyboard.just_pressed(KeyCode::ArrowUp) {
        state.item_count += 1;
        for mut t in &mut plural_query {
            *t = I18nMarker::plural("player.inventory", state.item_count);
        }
    }
    if keyboard.just_pressed(KeyCode::ArrowDown) {
        state.item_count = state.item_count.saturating_sub(1);
        for mut t in &mut plural_query {
            *t = I18nMarker::plural("player.inventory", state.item_count);
        }
    }

    // Switch locale
    if keyboard.just_pressed(KeyCode::ArrowRight) {
        i18n.set_locale("zh");
    }
    if keyboard.just_pressed(KeyCode::ArrowLeft) {
        i18n.set_locale("en");
    }

    // Change score → TVar update propagates to T components automatically
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
}
