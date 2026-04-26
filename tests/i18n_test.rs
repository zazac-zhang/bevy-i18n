use bevy::asset::AssetServer;
use bevy::prelude::*;
use bevy_i18n::prelude::*;

fn setup_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin::default())
        .add_plugins(I18nPlugin);
    app
}

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

    // Run the app a few times to let assets load asynchronously
    for _ in 0..10 {
        app.update();
    }

    let locales = app.world().resource::<bevy::asset::Assets<I18nAsset>>();
    let i18n = app.world().resource::<I18n>();

    // Check English translation loaded
    let greeting = i18n.get("player.greeting", &[("name", "Hero")], &locales);
    assert_eq!(greeting, "Hello, Hero!");

    let title = i18n.get("game.title", &[], &locales);
    assert_eq!(title, "Star Trek");
}

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
    for _ in 0..10 {
        app.update();
    }

    // Switch to Chinese
    {
        let mut i18n = app.world_mut().resource_mut::<I18n>();
        i18n.set_locale("zh_CN");
    }

    // Run update to process locale change
    app.update();

    let locales = app.world().resource::<bevy::asset::Assets<I18nAsset>>();
    let i18n = app.world().resource::<I18n>();

    let title = i18n.get("game.title", &[], &locales);
    assert_eq!(title, "星际迷航");

    let greeting = i18n.get("player.greeting", &[("name", "张三")], &locales);
    assert_eq!(greeting, "你好，张三！");
}

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
    for _ in 0..10 {
        app.update();
    }

    // Check that Text was updated
    let mut world = app.world_mut();
    let text = world.entity(entity).get::<Text>().unwrap();
    assert_eq!(text.0, "Star Trek");
}
