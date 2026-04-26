# Examples

Each example demonstrates a different aspect of `bevy_i18n`. Run with:

```bash
cargo run --example <name>
```

## basic

Minimal setup: register the plugin, load locale files, and spawn translated text.

Demonstrates: `T::new()`, `T::with_vars()`, `T::plural()`

```bash
cargo run --example basic
```

## locale_switch

Runtime locale switching with keyboard input. All `T` components automatically update.

Demonstrates: `i18n.set_locale()`, `T::ns()`, locale change detection

```bash
cargo run --example locale_switch
# Press Space to toggle between English and Chinese
```

## advanced_features

Comprehensive demo of all features: context disambiguation, dynamic variables (`TVar`),
number/currency formatting, namespace builder, and missing key handling.

Demonstrates: `T::with_context()`, `T::with_dynamic_var()`, `NumberFormat`,
`T::ns()`, missing key fallbacks

```bash
cargo run --example advanced_features
# ↑↓ change count  ←→ switch locale  +- change score
```

## Locale files

Examples use locale files from `assets/locales/`. The supported keys are:

| Key | Description |
|-----|-------------|
| `game.title` | Simple text |
| `game.subtitle` | Simple text |
| `player.greeting` | With `{name}` variable |
| `player.score` | With `{score}` variable |
| `player.inventory.zero/one/other` | Plural forms |
| `menu.new_game`, `menu.settings`, `menu.quit` | Menu items |
| `menu.open` / `dialog::open` | Context disambiguation |
| `shop.price` | Currency formatting `{amount::currency}` |
| `locale.display` | With `{locale}` variable |
