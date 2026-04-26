# bevy_i18n

A lightweight internationalization plugin for Bevy 0.18.

## Features

- **YAML locale files** — simple `key: value` format with dot-notation nesting
- **Variable interpolation** — `{name}` placeholders in translations
- **Plural forms** — `zero`, `one`, `other` selection
- **Fallback locale** — automatic fallback when a key is missing
- **Hot reload** — edit YAML files and see changes in real-time
- **Per-locale fonts** — set different fonts for different locales
- **Number/currency formatting** — `{amount::number}`, `{price::currency}`
- **Context disambiguation** — same word, different translations
- **Namespacing** — `T::ns("ui.menu").key("quit")` builder pattern
- **Missing key warnings** — debug-mode alerts for untranslated keys
- **Translation cache** — fast lookups with automatic invalidation

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
bevy = "0.18"
bevy_i18n = { path = "path/to/bevy_I18n" }
```

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

Place YAML files in `assets/locales/`:

```yaml
# assets/locales/en.yaml
greeting: "Hello, {name}!"
game.title: "Star Trek"
items.count:
  zero: "No items"
  one: "One item"
  other: "{count} items"
price: "Price: {amount::currency}"
```

```yaml
# assets/locales/zh.yaml
greeting: "你好，{name}！"
game.title: "星际迷航"
items.count:
  zero: "没有物品"
  one: "一个物品"
  other: "{count} 个物品"
price: "价格: {amount::currency}"
```

### 3. Load locales and spawn text

```rust
use bevy::prelude::*;
use bevy_i18n::prelude::*;

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

    // Static text
    commands.spawn((
        Text::new(""),
        T::new("game.title"),
    ));

    // With variables
    commands.spawn((
        Text::new(""),
        T::with_vars("greeting", &[("name", "Player")]),
    ));

    // Plural forms
    let item_count = 5;
    commands.spawn((
        Text::new(""),
        T::plural("items.count", item_count),
    ));
}
```

### 4. Switch language at runtime

```rust
fn switch_to_chinese(mut i18n: ResMut<I18n>) {
    i18n.set_locale("zh");
    // All T components automatically update
}
```

## API Reference

### T Component

| Constructor | Description |
|-------------|-------------|
| `T::new(key)` | Simple key lookup |
| `T::with_vars(key, &[("var", "value")])` | Key with variable substitutions |
| `T::plural(key, count)` | Key with plural form selection |
| `T::with_context(key, context)` | Key with context disambiguation |
| `T::ns("namespace").key(subkey)` | Namespaced lookup (`namespace.subkey`) |

### I18n Resource

| Method | Description |
|--------|-------------|
| `add_locale(locale, handle)` | Register a locale with its asset handle |
| `set_locale(locale)` | Switch the active locale |
| `set_fallback_locale(locale)` | Set fallback for missing keys |
| `set_locale_font(handle)` | Set font for current locale |
| `set_locale_number_format(locale, format)` | Set number formatting rules |
| `set_missing_key_config(config)` | Configure missing key warnings |
| `reset_missing_key_count()` | Reset and return missing key counter |
| `get(key, vars, assets)` | Look up a translation |
| `is_locale_loaded(locale, assets)` | Check if a locale is loaded |

### NumberFormat

```rust
use bevy_i18n::NumberFormat;

i18n.set_locale_number_format("en", NumberFormat {
    thousands_sep: ',',
    decimal_sep: '.',
    decimal_places: Some(2),
    currency_symbol: Some("$".to_string()),
});
```

## Format Specifiers

Translations can use format specifiers for automatic number/currency formatting:

```yaml
# YAML
balance: "Balance: {amount::currency}"
score: "Score: {points::number}"
```

```rust
// Set up the format rules
i18n.set_locale_number_format("en", NumberFormat {
    thousands_sep: ',',
    decimal_sep: '.',
    decimal_places: Some(2),
    currency_symbol: Some("$".to_string()),
});

// Results in: "Balance: $ 1,234.50"
// Results in: "Score: 1,000.00"
```

## CLI Tools

### Extract keys from source code

```bash
# Scan src/ and generate template
cargo run --bin i18n-extract -- src/

# Output to a specific file
cargo run --bin i18n-extract -- src/ locales/template.yaml
```

### Validate locale consistency

```bash
# Check all locales have the same keys
cargo run --bin i18n-validate -- assets/locales

# Returns non-zero exit code if issues found (useful for CI)
```

## Project Structure

```
src/
  lib.rs          - Module declarations and prelude
  asset.rs        - I18nAsset type and YAML/PO loaders
  component.rs    - T component and NamespaceBuilder
  interpolate.rs  - Variable interpolation and NumberFormat
  plugin.rs       - I18nPlugin registration
  resource.rs     - I18n resource (locale management)
  systems.rs      - Translation resolution and hot reload systems
  bin/
    i18n-extract.rs   - Key extraction CLI tool
    i18n-validate.rs  - Locale validation CLI tool
```

## License

MIT OR Apache-2.0
