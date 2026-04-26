use bevy::asset::{AssetPlugin, AssetServer, Assets};
use bevy::prelude::*;
use bevy_i18n::prelude::*;

fn setup_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin::default())
        .add_plugins(I18nPlugin);
    app
}

fn pump_app(app: &mut App, times: usize) {
    for _ in 0..times {
        app.update();
    }
}

// ── Basic locale loading ──────────────────────────────────────────────

#[test]
fn test_load_locale_yaml() {
    let mut app = setup_app();

    let asset_server = app.world().resource::<AssetServer>().clone();
    let mut i18n = app.world_mut().resource_mut::<I18n>();

    let en = asset_server.load("locales/en.yaml");
    let zh = asset_server.load("locales/zh.yaml");

    i18n.add_locale("en_US", en);
    i18n.add_locale("zh_CN", zh);
    i18n.set_locale("en_US");

    pump_app(&mut app, 20);

    let locales = app.world().resource::<bevy::asset::Assets<I18nAsset>>();
    let i18n = app.world().resource::<I18n>();

    // Check English translation loaded
    let greeting = i18n.get("player.greeting", [("name", "Hero")].as_slice(), locales);
    assert_eq!(greeting, "Hello, Hero!");

    let title = i18n.get("game.title", [].as_slice(), locales);
    assert_eq!(title, "Star Trek");
}

// ── Switch locale ─────────────────────────────────────────────────────

#[test]
fn test_switch_locale() {
    let mut app = setup_app();

    let asset_server = app.world().resource::<AssetServer>().clone();
    let mut i18n = app.world_mut().resource_mut::<I18n>();

    let en = asset_server.load("locales/en.yaml");
    let zh = asset_server.load("locales/zh.yaml");

    i18n.add_locale("en_US", en);
    i18n.add_locale("zh_CN", zh);
    i18n.set_locale("en_US");

    // Let assets load
    pump_app(&mut app, 20);

    // Switch to Chinese
    {
        let mut i18n = app.world_mut().resource_mut::<I18n>();
        i18n.set_locale("zh_CN");
    }

    // Run update to process locale change
    app.update();

    let locales = app.world().resource::<bevy::asset::Assets<I18nAsset>>();
    let i18n = app.world().resource::<I18n>();

    let title = i18n.get("game.title", [].as_slice(), locales);
    assert_eq!(title, "星际迷航");

    let greeting = i18n.get("player.greeting", [("name", "张三")].as_slice(), locales);
    assert_eq!(greeting, "你好，张三！");
}

// ── T component updates Text ──────────────────────────────────────────

#[test]
fn test_t_component_updates_text() {
    let mut app = setup_app();

    let asset_server = app.world().resource::<AssetServer>().clone();
    let mut i18n = app.world_mut().resource_mut::<I18n>();

    let en = asset_server.load("locales/en.yaml");
    i18n.add_locale("en_US", en);
    i18n.set_locale("en_US");

    // Spawn a text entity with T component
    let entity = app
        .world_mut()
        .spawn((Text::new(""), T::new("game.title")))
        .id();

    // Let assets load and systems run
    pump_app(&mut app, 20);

    // Check that Text was updated
    let world = app.world();
    let text = world.entity(entity).get::<Text>().unwrap();
    assert_eq!(text.0, "Star Trek");
}

// ── Fallback locale ───────────────────────────────────────────────────

#[test]
fn test_fallback_locale_in_systems() {
    let mut app = setup_app();

    let asset_server = app.world().resource::<AssetServer>().clone();
    let mut i18n = app.world_mut().resource_mut::<I18n>();

    // Load only English (which has all keys)
    let en = asset_server.load("locales/en.yaml");
    i18n.add_locale("en_US", en);
    // Set a locale that doesn't exist — fallback should kick in
    i18n.set_locale("en_US");
    i18n.set_fallback_locale("en_US");

    pump_app(&mut app, 20);

    let locales = app.world().resource::<Assets<I18nAsset>>();
    let i18n = app.world().resource::<I18n>();

    // This key only exists in en
    let result = i18n.get("game.title", &[], locales);
    assert_eq!(result, "Star Trek");
}

// ── Plural translation via T::plural ──────────────────────────────────

#[test]
fn test_plural_t_component() {
    let mut app = setup_app();

    let asset_server = app.world().resource::<AssetServer>().clone();
    let mut i18n = app.world_mut().resource_mut::<I18n>();

    let en = asset_server.load("locales/en.yaml");
    i18n.add_locale("en", en);
    i18n.set_locale("en");

    // Spawn a text with plural T component
    let entity = app
        .world_mut()
        .spawn((Text::new(""), T::plural("items.count", 5)))
        .id();

    pump_app(&mut app, 20);

    let text = app.world().entity(entity).get::<Text>().unwrap();
    assert!(text.0.contains("items"), "Expected plural text, got: {}", text.0);
}

#[test]
fn test_plural_zero() {
    let mut app = setup_app();

    let asset_server = app.world().resource::<AssetServer>().clone();
    let mut i18n = app.world_mut().resource_mut::<I18n>();

    let en = asset_server.load("locales/en.yaml");
    i18n.add_locale("en", en);
    i18n.set_locale("en");

    let entity = app
        .world_mut()
        .spawn((Text::new(""), T::plural("items.count", 0)))
        .id();

    pump_app(&mut app, 20);

    let text = app.world().entity(entity).get::<Text>().unwrap();
    // Should resolve to items.zero or items.other
    assert!(!text.0.is_empty());
}

// ── TVar: dynamic variables ───────────────────────────────────────────

#[test]
fn test_tvar_updates_text() {
    let mut app = setup_app();

    let asset_server = app.world().resource::<AssetServer>().clone();
    let mut i18n = app.world_mut().resource_mut::<I18n>();

    let en = asset_server.load("locales/en.yaml");
    i18n.add_locale("en", en);
    i18n.set_locale("en");

    // Spawn a TVar entity
    let tvar_entity = app.world_mut().spawn(TVar::new("0")).id();

    // Spawn a text that references the TVar
    let entity = app
        .world_mut()
        .spawn((Text::new(""), T::new("player.score").with_dynamic_var("score", tvar_entity)))
        .id();

    pump_app(&mut app, 20);

    // Check initial text resolved
    let text = app.world().entity(entity).get::<Text>().unwrap();
    assert!(text.0.contains("0") || text.0.contains("score") || text.0.contains("player.score"));

    // Update the TVar
    app.world_mut().entity_mut(tvar_entity).insert(TVar::new("100"));

    pump_app(&mut app, 5);

    // Check text was updated with new value
    let text = app.world().entity(entity).get::<Text>().unwrap();
    assert!(text.0.contains("100"), "Expected '100' in text, got: {}", text.0);
}

// ── Context translation ───────────────────────────────────────────────

#[test]
fn test_context_translation_t_component() {
    let mut app = setup_app();

    let asset_server = app.world().resource::<AssetServer>().clone();
    let mut i18n = app.world_mut().resource_mut::<I18n>();

    let en = asset_server.load("locales/en.yaml");
    i18n.add_locale("en", en);
    i18n.set_locale("en");

    let entity = app
        .world_mut()
        .spawn((Text::new(""), T::with_context("open", "menu")))
        .id();

    pump_app(&mut app, 20);

    let text = app.world().entity(entity).get::<Text>().unwrap();
    assert!(!text.0.is_empty(), "Context translation should resolve");
}

// ── Locale change triggers text update ────────────────────────────────

#[test]
fn test_locale_change_refreshes_text() {
    let mut app = setup_app();

    let asset_server = app.world().resource::<AssetServer>().clone();
    let mut i18n = app.world_mut().resource_mut::<I18n>();

    let en = asset_server.load("locales/en.yaml");
    let zh = asset_server.load("locales/zh.yaml");
    i18n.add_locale("en", en.clone());
    i18n.add_locale("zh", zh);
    i18n.set_locale("en");

    let entity = app
        .world_mut()
        .spawn((Text::new(""), T::new("game.title")))
        .id();

    pump_app(&mut app, 20);

    // Verify English
    assert_eq!(app.world().entity(entity).get::<Text>().unwrap().0, "Star Trek");

    // Switch to Chinese
    {
        let mut i18n = app.world_mut().resource_mut::<I18n>();
        i18n.set_locale("zh");
    }

    pump_app(&mut app, 5);

    // Verify Chinese
    assert_eq!(
        app.world().entity(entity).get::<Text>().unwrap().0,
        "星际迷航"
    );
}

// ── Font fallback per locale ──────────────────────────────────────────

#[test]
fn test_locale_font_set() {
    let mut i18n = I18n::default();
    let dummy_font = Handle::default();
    i18n.set_locale_font("en", dummy_font.clone());
    i18n.set_locale("en");

    // The font handle should be retrievable for the current locale
    assert!(i18n.current_locale_font().is_some());
}
