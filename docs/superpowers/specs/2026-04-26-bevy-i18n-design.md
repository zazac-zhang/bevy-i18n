---
name: bevy-i18n-design
description: Design for a Bevy 0.18 i18n plugin with asset-based locale loading, T component, and runtime language switching
type: project
---

# Bevy I18n - Design Specification

## Overview

A lightweight, asset-driven internationalization plugin for Bevy 0.18. Translation data is loaded via Bevy's native `Asset` system, supporting multiple formats through feature flags, with a `T` component for automatic UI text translation and runtime language switching.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        I18nPlugin                           │
│  Registers: Assets, AssetLoaders, Systems, Resources         │
└──────────────────────┬──────────────────────────────────────┘
                       │
       ┌───────────────┼───────────────┐
       ▼               ▼               ▼
┌─────────────┐ ┌─────────────┐ ┌──────────────┐
│ I18nAsset   │ │I18nLoader   │ │ I18nResource │
│ (Asset)     │ │(AssetLoader)│ │  - locale     │
│ Translations│ │  YAML(default)│  - locale_map │
│ as HashMap  │ │  .po(feature)│ │  - hot-reload│
└─────────────┘ └─────────────┘ └──────┬───────┘
                                       │
                                       ▼
                              ┌────────────────┐
                              │ T(Component)   │
                              │  key + vars    │
                              │  auto-refresh  │
                              └────────────────┘
```

## Core Modules

### 1. `I18nAsset` - Custom Asset Type

Holds parsed translation data for a single locale.

```rust
pub struct I18nAsset {
    entries: HashMap<String, TransEntry>,
}

pub enum TransEntry {
    Static(String),
    Plural {
        zero: Option<String>,
        one: Option<String>,
        other: String,
    },
}
```

### 2. `I18nLoader` - AssetLoader

Parses translation files into `I18nAsset`.

- **Default**: YAML format via `serde_yaml`
- **Feature `po`**: `.po` format via `gettext` crate
- Registered automatically by `I18nPlugin`

### 3. `I18n` - Resource

Central state manager for the i18n system.

```rust
pub struct I18n {
    current_locale: String,
    fallback_locale: Option<String>,
    locale_map: HashMap<String, Handle<I18nAsset>>,
    parsed_locales: HashMap<String, Arc<I18nAsset>>,
}
```

Key methods:
- `set_locale(&mut self, locale: &str)` - Switch language, triggers refresh
- `add_locale(&mut self, locale: &str, handle: Handle<I18nAsset>)` - Register locale
- `get(&self, key: &str) -> Option<String>` - Look up translation
- `current_locale(&self) -> &str` - Get current language

### 4. `T` - Component

Marks a `Text` entity for automatic translation.

```rust
pub struct T {
    key: String,
    vars: Vec<(String, String)>,
    count: Option<u64>,  // for plural support
}
```

Usage:
- `T::new("game.title")` - Static translation
- `T::with_vars("player.greeting", &[("name", "张三")])` - With variables
- `T::plural("item.count", 5)` - Plural form

### 5. Systems

- `refresh_text_system` - Runs when locale changes, updates all `Text` entities with `T` components
- `init_t_text_system` - On first spawn, resolves translation key and sets `Text` content

## File Formats

### YAML (default)

```yaml
game:
  title: "星际迷航"
player:
  greeting: "你好，{name}！"
  inventory:
    zero: "背包空空如也"
    one: "{count} 个物品"
    other: "{count} 个物品"
```

Nested keys flatten to dot-notation: `game.title`, `player.greeting`, `player.inventory`.

### .po (feature: po)

Standard gettext `.po` format. `msgid` maps to key, `msgstr` to translation. Plurals via `msgstr[n]`.

## Variable Interpolation

Uses `{name}` syntax. Processed at lookup time:

```rust
fn interpolate(template: &str, vars: &[(String, String)]) -> String
```

`{count}` is a special variable for plural forms. If `count` is provided, the correct plural form is selected before interpolation.

## Locale Change Flow

```
i18n.set_locale("zh_CN")
  │
  ├─▶ Sends LocaleChanged event
  │
  ├─▶ refresh_text_system observes event
  │   └─▶ For each (T, Text) entity:
  │       ├─▶ Lookup key in zh_CN locale
  │       ├─▶ Apply variables
  │       └─▶ Update Text content
  │
  └─▶ Handles unloaded assets gracefully
      (Text shows key until asset loads)
```

## Plugin Registration

```rust
pub struct I18nPlugin;

impl Plugin for I18nPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<I18nAsset>()
           .init_asset_loader::<I18nLoader>()
           .init_resource::<I18n>()
           .add_event::<LocaleChanged>()
           .add_systems(
               Update,
               (refresh_text_system, init_t_text_system),
           );
    }
}
```

## Cargo.toml

```toml
[package]
name = "bevy_i18n"
version = "0.1.0"
edition = "2024"

[dependencies]
bevy = { version = "0.18", default-features = false, features = ["std"] }
serde = { version = "1", features = ["derive"] }
serde_yaml = "0.9"

[features]
default = ["yaml"]
yaml = ["dep:serde_yaml"]
po = ["dep:gettext"]  # placeholder: exact crate TBD
```

## Future Work (Out of Scope)

- CLI extraction tool (`cargo i18n-extract`) - reserved API hook
- RTL text support
- Font fallback per locale
- Translation validation/warnings for missing keys
