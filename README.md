# bevy_i18n

A lightweight internationalization plugin for Bevy 0.18.

## Features

- **YAML locale files** — simple `key: value` format with dot-notation nesting
- **Variable interpolation** — `{name}` placeholders in translations
- **Plural forms** — `zero`, `one`, `other` selection
- **Fallback locale** — automatic fallback when a key is missing
- **Hot reload** — edit YAML files and see changes in real-time
- **Per-locale fonts** — set different fonts for different locales
- **Number/currency formatting** — `{amount::currency}`, `{price::number}`
- **Context disambiguation** — same word, different translations (`msgctxt`)
- **Namespacing** — `I18nMarker::ns("ui.menu").key("quit")` builder pattern
- **Dynamic variables** — `TVar` component for runtime-updating values
- **Missing key warnings** — debug-mode alerts for untranslated keys
- **CLI tools** — extract keys from source, validate locale consistency
- **Derive macro** — `#[derive(I18n)]` for custom components (optional feature)

## Quick Start

### 1. Add the plugin

```rust
use bevy::prelude::*;
use bevy_i18n::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(I18nPlugin)
        .run();
}
```

### 2. Create locale files

```yaml
# assets/locales/en.yaml
greeting: "Hello, {name}!"
game.title: "Star Trek"
player.inventory:
  zero: "No items"
  one: "One item"
  other: "{count} items"
price: "Price: {amount::currency}"
```

```yaml
# assets/locales/zh.yaml
greeting: "你好，{name}！"
game.title: "星际迷航"
player.inventory:
  zero: "没有物品"
  one: "一个物品"
  other: "{count} 个物品"
price: "价格: {amount::currency}"
```

### 3. Load locales and spawn text

```rust
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

    // Simple text
    commands.spawn((Text::new(""), I18nMarker::new("game.title")));

    // With variables
    commands.spawn((
        Text::new(""),
        I18nMarker::with_vars("greeting", &[("name", "Player")]),
    ));

    // Plural forms
    commands.spawn((
        Text::new(""),
        I18nMarker::plural("player.inventory", 5),
    ));
}
```

### 4. Switch language at runtime

```rust
fn switch_to_chinese(mut i18n: ResMut<I18n>) {
    i18n.set_locale("zh");
    // All I18nMarker components automatically update
}
```

## API Reference

### I18nMarker Component

| Constructor | Description |
|-------------|-------------|
| `I18nMarker::new(key)` | Simple key lookup |
| `I18nMarker::marker()` | Empty marker — all `Localizable` fields translated |
| `I18nMarker::with_vars(key, &[("var", "value")])` | Key with variable substitutions |
| `I18nMarker::plural(key, count)` | Key with plural form selection |
| `I18nMarker::with_context(key, context)` | Key with context disambiguation |
| `I18nMarker::ns("namespace").key(subkey)` | Namespaced lookup (`namespace.subkey`) |
| `I18nMarker::ns("ns").with_vars(key, vars)` | Namespace with variables |

### Dynamic Variables (TVar)

For values that change at runtime (score, timer, etc.):

```rust
// Spawn a TVar entity
let score_entity = commands.spawn(TVar::new("0")).id();

// Reference it from a text entity
commands.spawn((
    Text::new(""),
    I18nMarker::new("player.score").with_dynamic_var("score", score_entity),
));

// Update the TVar value — text auto-updates
if let Ok(mut tvar) = score_query.get_mut(score_entity) {
    tvar.value = "1000".to_string();
}
```

### I18n Resource

| Method | Description |
|--------|-------------|
| `add_locale(locale, handle)` | Register a locale with its asset handle |
| `set_locale(locale)` | Switch the active locale |
| `set_fallback_locale(locale)` | Set fallback for missing keys |
| `set_locale_font(handle)` | Set font for current locale |
| `set_locale_number_format(locale, format)` | Set number formatting rules |
| `get(key, vars, assets)` | Look up a translation directly |

### NumberFormat

```rust
i18n.set_locale_number_format("en", NumberFormat {
    thousands_sep: ',',
    decimal_sep: '.',
    decimal_places: Some(2),
    currency_symbol: Some("$".to_string()),
});
```

### Custom Components (Localizable trait)

To translate custom text components, implement `Localizable`:

```rust
#[derive(Component)]
struct CustomText { content: String }

impl Localizable for CustomText {
    fn translations() -> &'static [(&'static str, &'static str)] {
        &[("content", "custom.message")]
    }
    fn set_field(&mut self, field: &str, value: &str) {
        if field == "content" { self.content = value.into(); }
    }
}
```

Then register the generic update system:

```rust
app.add_systems(Update, update_localizable::<CustomText>);
```

And spawn with `I18nMarker`:

```rust
// Single-field component
commands.spawn((
    CustomText::default(),
    I18nMarker::new("custom.message"),
));

// Multi-field component — all fields translated via translations()
commands.spawn((
    I18nMarker::marker(),
    DialogBox {
        title: String::new(),
        body: String::new(),
    },
));
```

### Derive Macro (optional)

Enable with `features = ["derive"]`:

```rust
use bevy::prelude::*;
use bevy_i18n::prelude::*;

#[derive(I18n, Component)]
struct DialogBox {
    #[i18n(key = "dialog.title")]
    title: String,
    #[i18n(key = "dialog.body")]
    body: String,
    color: Color, // non-String field, auto-ignored
}

// With namespace
#[derive(I18n, Component)]
#[i18n(namespace = "hud")]
struct HUD {
    #[i18n(key = "score")]   // → "hud.score"
    score_text: String,
}
```

## How It Works

`I18nMarker::new()` does **not** translate — it creates a marker component with `dirty: true`.
The `update_localizable::<Text>` system runs every frame, finds all entities with both `I18nMarker` and `Text`,
translates the dirty ones, and clears the flag. Language changes set all markers dirty again,
triggering automatic re-translation.

For custom `Localizable` components, register `update_localizable::<YourComponent>` and spawn
with `I18nMarker::marker()` — all fields from `YourComponent::translations()` are translated automatically.

```
commands.spawn((Text::new(""), I18nMarker::new("key")))
    → Entity { Text(""), I18nMarker { dirty: true } }
    → update_localizable::<Text> finds (I18nMarker, Text) pair
    → translates and sets Text content, dirty = false
    → on locale change, all dirty = true → re-translate
```

## CLI Tools

### Extract keys from source code

```bash
cargo run --bin i18n-extract -- src/
cargo run --bin i18n-extract -- src/ locales/template.yaml
```

### Validate locale consistency

```bash
cargo run --bin i18n-validate -- assets/locales
# Returns non-zero exit code if issues found (useful for CI)
```

## Project Structure

```
src/
  lib.rs          - Module declarations and prelude
  asset.rs        - I18nAsset type and YAML/PO loaders
  component.rs    - I18nMarker / TVar components, Localizable trait
  interpolate.rs  - Variable interpolation and NumberFormat
  plugin.rs       - I18nPlugin registration
  resource.rs     - I18n resource (locale management)
  systems.rs      - Translation resolution and hot reload systems
  bin/
    i18n-extract.rs   - Key extraction CLI
    i18n-validate.rs  - Locale validation CLI
derive/
  src/lib.rs      - #[derive(I18n)] proc-macro
  src/attr.rs     - Attribute parsing
```

## Examples

| Example | Description | Run |
|---------|-------------|-----|
| `basic` | Minimal setup — load locales, spawn text | `cargo run --example basic` |
| `locale_switch` | Press Space to toggle between languages | `cargo run --example locale_switch` |
| `advanced_features` | All features: plural, context, TVar, formatting | `cargo run --example advanced_features` |
| `custom_component` | `#[derive(I18n)]` for custom components | `cargo run --example custom_component --features derive` |

## License

MIT OR Apache-2.0
