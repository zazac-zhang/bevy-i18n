# Restructure & Derive Macro Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Consolidate broken derive/macro directories into a workspace with optional derive feature, unify the Localizable trait, and enhance the derive macro with multi-field and namespace support.

**Architecture:** Create a Cargo workspace with the main `bevy_i18n` crate and a new `bevy_i18n_derive` proc-macro crate. Replace the old `Localizable` trait with a unified `translations()` + `set_field()` design. The derive macro supports `#[i18n(key = "...")]`, `#[i18n]`, `#[i18n(skip)]`, and struct-level `#[i18n(namespace = "...")]`.

**Tech Stack:** Rust, Bevy 0.18, proc-macro2/syn/quote, Cargo workspaces

---

### Task 1: Create derive crate Cargo.toml

**Files:**
- Create: `derive/Cargo.toml`

- [ ] **Step 1: Write derive/Cargo.toml**

```toml
[package]
name = "bevy_i18n_derive"
version = "0.1.0"
edition = "2021"
description = "Derive macro for bevy_i18n Localizable trait"
license = "MIT OR Apache-2.0"

[lib]
proc-macro = true

[dependencies]
syn = "2.0"
quote = "1.0"
proc-macro2 = "1.0"
```

- [ ] **Step 2: Commit**

```bash
git add derive/Cargo.toml
git commit -m "feat: add bevy_i18n_derive Cargo.toml"
```

---

### Task 2: Delete empty bevy_i18n_macro directory

**Files:**
- Delete: `bevy_i18n_macro/` (entire directory)

- [ ] **Step 1: Remove directory**

```bash
rm -rf bevy_i18n_macro/
```

- [ ] **Step 2: Commit**

```bash
git add -u bevy_i18n_macro/
git commit -m "chore: remove empty bevy_i18n_macro directory"
```

---

### Task 3: Set up Cargo workspace

**Files:**
- Modify: `Cargo.toml` (root)

- [ ] **Step 1: Rewrite root Cargo.toml to be a workspace**

The current Cargo.toml is a single `[package]`. Replace it with a workspace that includes both the main crate and the derive crate.

```toml
[workspace]
members = [".", "derive"]

[package]
name = "bevy_i18n"
version = "0.1.0"
edition = "2024"
description = "A lightweight i18n plugin for Bevy 0.18"
license = "MIT OR Apache-2.0"

[dependencies]
bevy = { version = "0.18", default-features = false, features = ["std", "bevy_asset", "bevy_text", "bevy_ui"] }
regex = "1"
serde = { version = "1", features = ["derive"] }
serde_yaml = { version = "0.9", optional = true }
bevy_i18n_derive = { version = "0.1.0", path = "derive", optional = true }

[features]
default = ["yaml"]
yaml = ["dep:serde_yaml"]
po = []
derive = ["dep:bevy_i18n_derive"]

[dev-dependencies]
bevy = { version = "0.18", default-features = false, features = ["std", "bevy_asset", "bevy_text", "bevy_ui"] }
divan = "0.1"

[[bench]]
name = "i18n_bench"
harness = false

[[example]]
name = "basic"
path = "examples/basic.rs"

[[example]]
name = "locale_switch"
path = "examples/locale_switch.rs"

[[example]]
name = "advanced_features"
path = "examples/advanced_features.rs"
```

- [ ] **Step 2: Commit**

```bash
git add Cargo.toml
git commit -m "feat: set up cargo workspace with optional derive feature"
```

---

### Task 4: Rewrite Localizable trait (unified design)

**Files:**
- Modify: `src/component.rs` — replace `Localizable` trait, remove old `I18nKey`
- Modify: `src/systems.rs` — update systems to use new trait
- Modify: `src/lib.rs` — update prelude exports
- Modify: `tests/i18n_test.rs` — update tests for new API

The unified `Localizable` trait replaces both the old `Localizable` (single field, `set_text`) and the planned `MultiLocalizable`. All derive-generated code uses one trait.

- [ ] **Step 1: Replace Localizable trait in src/component.rs**

Replace the existing `Localizable` trait (lines 28-31) and `impl Localizable for Text` (lines 34-38) with the new unified design:

```rust
/// Trait for components that can be internationalized.
///
/// Implement this trait to support automatic translation updates
/// on any component type. The derive macro `#[derive(I18n)]` generates
/// this implementation automatically.
///
/// # Example (manual implementation)
/// ```ignore
/// #[derive(Component)]
/// struct MyLabel { text: String }
///
/// impl Localizable for MyLabel {
///     fn translations() -> &'static [(&'static str, &'static str)] {
///         &[("text", "label.hello")]
///     }
///     fn set_field(&mut self, field: &str, value: &str) {
///         if field == "text" { self.text = value.into(); }
///     }
/// }
/// ```
pub trait Localizable: Component {
    /// Returns `(field_name, translation_key)` pairs for all translatable fields.
    fn translations() -> &'static [(&'static str, &'static str)];
    /// Set a translatable field by name.
    fn set_field(&mut self, field_name: &str, value: &str);
}

/// Default implementation for Bevy's Text component.
/// Used internally by `update_text_system`.
impl Localizable for Text {
    fn translations() -> &'static [(&'static str, &'static str)] {
        &[("0", "_text_")]
    }
    fn set_field(&mut self, _field_name: &str, value: &str) {
        self.0 = value.to_string();
    }
}
```

- [ ] **Step 2: Remove the old I18nKey struct from src/component.rs**

Delete lines 262-347 (the entire `I18nKey` struct and its impl block). `I18nKey` is replaced by the `translations()` mechanism — custom components use `#[derive(I18n)]` instead of `(I18nKey, Text)` tuples.

- [ ] **Step 3: Update src/lib.rs prelude**

Replace the prelude module:

```rust
mod asset;
mod component;
pub mod interpolate;
mod plugin;
mod resource;
pub mod systems;

/// Prelude — one-import convenience
pub mod prelude {
    pub use crate::asset::I18nAsset;
    pub use crate::component::{I18nText, Localizable, TVar};
    pub use crate::interpolate::NumberFormat;
    pub use crate::plugin::I18nPlugin;
    pub use crate::resource::I18n;
    pub use crate::systems::{update_i18n, update_localizable, update_text_system};

    #[cfg(feature = "derive")]
    pub use bevy_i18n_derive::I18n;
}

pub use crate::interpolate::NumberFormat;

#[cfg(feature = "derive")]
pub use bevy_i18n_derive::I18n;
```

- [ ] **Step 4: Commit**

```bash
git add src/component.rs src/lib.rs
git commit -m "refactor: unify Localizable trait with translations()/set_field() API"
```

---

### Task 5: Update systems to use new Localizable trait

**Files:**
- Modify: `src/systems.rs`
- Modify: `src/plugin.rs`

- [ ] **Step 1: Rewrite src/systems.rs**

Replace the entire file with updated systems that use the new `Localizable` trait API:

```rust
use bevy::asset::{AssetEvent, Assets};
use bevy::ecs::component::Mutable;
use bevy::ecs::message::MessageReader;
use bevy::prelude::*;
use bevy::text::TextFont;
use bevy::ui::prelude::Text;

use crate::asset::I18nAsset;
use crate::component::{I18nText, Localizable, TVar};
use crate::resource::I18n;

/// Resolves translation keys for all I18nText components.
pub fn resolve_translations(
    mut i18n: ResMut<I18n>,
    mut query: Query<&mut I18nText>,
) {
    let locale_changed = i18n.update_from();
    if locale_changed {
        for mut t in &mut query {
            t.mark_dirty();
        }
    }
}

/// Checks dynamic variable references on I18nText components.
pub fn resolve_dynamic_vars(
    mut query: Query<&mut I18nText>,
    tvar_query: Query<&TVar>,
) {
    for mut t in &mut query {
        if t.dynamic_vars.is_empty() {
            continue;
        }
        let current_values: Vec<(String, String)> = t
            .dynamic_vars
            .iter()
            .filter_map(|(name, entity)| {
                tvar_query.get(*entity).ok().map(|tv| (name.clone(), tv.value.clone()))
            })
            .collect();
        if current_values.is_empty() {
            continue;
        }
        let last_values = t
            .vars
            .iter()
            .filter(|(k, _)| t.dynamic_vars.iter().any(|(dn, _)| dn == k))
            .cloned()
            .collect::<Vec<_>>();
        if current_values != last_values {
            t.mark_dirty();
        }
    }
}

/// Updates Text content for dirty I18nText components.
pub fn update_text_system(
    i18n: Res<I18n>,
    locales: Res<Assets<I18nAsset>>,
    mut query: Query<(&mut I18nText, &mut Text)>,
    tvar_query: Query<&TVar>,
    mut text_fonts: Query<&mut TextFont, With<I18nText>>,
) {
    if let Some(font_handle) = i18n.current_locale_font() {
        for mut text_font in &mut text_fonts {
            text_font.font = font_handle.clone();
        }
    }
    for (mut t, mut text) in &mut query {
        if !t.dirty {
            continue;
        }
        let mut all_vars: Vec<(String, String)> = t.vars.clone();
        for (name, entity) in &t.dynamic_vars {
            if let Ok(tvar) = tvar_query.get(*entity) {
                all_vars.push((name.clone(), tvar.value.clone()));
            }
        }
        let vars: Vec<(&str, &str)> =
            all_vars.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect::<Vec<_>>();
        let translated = i18n.get_plural(&t.key, t.context.as_deref(), t.count, &vars, &locales);
        text.0 = translated;
        t.dirty = false;
    }
}

/// Listens for I18nAsset changes and marks all I18nText components dirty on reload.
pub fn hot_reload_system(
    mut events: MessageReader<AssetEvent<I18nAsset>>,
    mut i18n: ResMut<I18n>,
    mut query: Query<&mut I18nText>,
) {
    for event in events.read() {
        if matches!(
            event,
            AssetEvent::LoadedWithDependencies { .. } | AssetEvent::Modified { .. }
        ) {
            i18n.clear_translation_cache();
            for mut t in &mut query {
                t.mark_dirty();
            }
        }
    }
}

/// Generic update system for any `Localizable` component paired with `I18nText`.
///
/// Register with `app.add_systems(Update, update_localizable::<MyComponent>)`.
pub fn update_localizable<T: Localizable + Component<Mutability = Mutable>>(
    i18n: Res<I18n>,
    locales: Res<Assets<I18nAsset>>,
    mut query: Query<(&mut I18nText, &mut T)>,
) {
    for (mut t, mut component) in query.iter_mut() {
        if !t.dirty {
            continue;
        }
        let vars: Vec<(&str, &str)> = t
            .vars
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        let translation = i18n.get_plural(
            &t.key,
            t.context.as_deref(),
            t.count,
            &vars,
            &locales,
        );
        // Find the matching field and set it
        for (field_name, _key) in T::translations() {
            component.set_field(field_name, &translation);
        }
        t.dirty = false;
    }
}

/// Alias for update_localizable for backwards compatibility.
pub fn update_i18n<T: Localizable + Component<Mutability = Mutable>>(
    i18n: Res<I18n>,
    locales: Res<Assets<I18nAsset>>,
    query: Query<(&mut I18nText, &mut T)>,
) {
    update_localizable::<T>(i18n, locales, query)
}
```

Key changes:
- Removed `I18nKey` import and usage (replaced by `I18nText`)
- `update_localizable` now queries `(&mut I18nText, &mut T)` instead of `(&mut I18nKey, &mut T)`
- Uses `T::translations()` to iterate all translatable fields

- [ ] **Step 2: Update src/plugin.rs imports**

The plugin.rs imports don't need changes — `update_localizable` and `update_text_system` are still exported from systems. No changes needed.

- [ ] **Step 3: Verify compilation**

```bash
cargo check --lib
```

Expected: clean compilation with no errors.

- [ ] **Step 4: Update tests to remove I18nKey references**

Read `tests/i18n_test.rs` — it doesn't use `I18nKey` directly, only `I18nText`, `TVar`, `I18n`, `I18nPlugin`, `Text`. No changes needed to tests.

- [ ] **Step 5: Commit**

```bash
git add src/systems.rs
git commit -m "refactor: update systems to use unified Localizable trait"
```

---

### Task 6: Rewrite derive macro with full feature set

**Files:**
- Rewrite: `derive/src/lib.rs`
- Create: `derive/src/attr.rs`

- [ ] **Step 1: Write derive/src/attr.rs — attribute parsing**

```rust
use syn::{Attribute, Meta, Expr, ExprLit, Lit, LitStr, Error, Result};
use proc_macro2::Span;

/// Parsed representation of a single field's #[i18n(...)] attribute.
#[derive(Default)]
pub struct FieldI18nAttr {
    pub key: Option<String>,
    pub skip: bool,
}

/// Parsed representation of struct-level #[i18n(...)] attributes.
#[derive(Default)]
pub struct StructI18nConfig {
    pub namespace: Option<String>,
}

/// Parse struct-level #[i18n(namespace = "...")] attribute.
pub fn parse_struct_attrs(attrs: &[Attribute]) -> Result<StructI18nConfig> {
    let mut config = StructI18nConfig::default();
    for attr in attrs {
        if !attr.path().is_ident("i18n") {
            continue;
        }
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("namespace") {
                let value = meta.value()?;
                let lit: LitStr = value.parse()?;
                config.namespace = Some(lit.value());
                Ok(())
            } else {
                Err(meta.error("unsupported struct-level i18n attribute"))
            }
        })?;
    }
    Ok(config)
}

/// Parse a field's #[i18n], #[i18n(key = "...")], or #[i18n(skip)].
/// Returns Some(attr) if the field has an i18n attribute, None otherwise.
pub fn parse_field_attrs(attrs: &[Attribute]) -> Result<Option<FieldI18nAttr>> {
    let mut result = FieldI18nAttr::default();
    let mut found = false;

    for attr in attrs {
        if !attr.path().is_ident("i18n") {
            continue;
        }
        found = true;

        // Handle bare #[i18n] — no args, means "use field name as key"
        if matches!(attr.meta, Meta::Path(_)) {
            result.key = None; // will be set to field name later
            continue;
        }

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("key") {
                let value = meta.value()?;
                let lit: LitStr = value.parse()?;
                result.key = Some(lit.value());
                Ok(())
            } else if meta.path.is_ident("skip") {
                result.skip = true;
                Ok(())
            } else {
                Err(meta.error("unsupported i18n field attribute, expected `key` or `skip`"))
            }
        })?;
    }

    if found {
        Ok(Some(result))
    } else {
        Ok(None)
    }
}
```

- [ ] **Step 2: Rewrite derive/src/lib.rs**

```rust
//! Derive macro for bevy_i18n Localizable trait.
//!
//! # Usage
//!
//! ```ignore
//! use bevy::prelude::*;
//! use bevy_i18n::prelude::*;
//!
//! #[derive(I18n, Component)]
//! struct DialogBox {
//!     #[i18n(key = "dialog.title")]
//!     title: String,
//!     #[i18n(key = "dialog.body")]
//!     content: String,
//!     color: Color, // non-String field, auto-ignored
//! }
//!
//! // With namespace
//! #[derive(I18n, Component)]
//! #[i18n(namespace = "hud")]
//! struct HUD {
//!     #[i18n(key = "score")]   // → "hud.score"
//!     score_text: String,
//! }
//! ```

mod attr;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, DataStruct, Fields};
use attr::{parse_struct_attrs, parse_field_attrs};

#[proc_macro_derive(I18n, attributes(i18n))]
pub fn derive_i18n(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let struct_config = match parse_struct_attrs(&input.attrs) {
        Ok(c) => c,
        Err(e) => return e.to_compile_error().into(),
    };

    let fields = match &input.data {
        Data::Struct(DataStruct { fields: Fields::Named(f), .. }) => &f.named,
        _ => {
            return quote! {
                compile_error!("#[derive(I18n)] only supports structs with named fields");
            }
            .into();
        }
    };

    let mut translatable_fields: Vec<(syn::Ident, String)> = Vec::new();

    for field in fields {
        let field_ident = field.ident.as_ref().unwrap();

        // Skip non-String fields
        let is_string = if let syn::Type::Path(type_path) = &field.ty {
            type_path
                .path
                .segments
                .last()
                .map(|s| s.ident == "String")
                .unwrap_or(false)
        } else {
            false
        };

        let field_attr = match parse_field_attrs(&field.attrs) {
            Ok(a) => a,
            Err(e) => return e.to_compile_error().into(),
        };

        match field_attr {
            Some(attr) if attr.skip => {
                // Explicitly skipped
                continue;
            }
            Some(attr) => {
                // Has #[i18n] — use key if provided, else field name
                let key = attr.key.unwrap_or_else(|| field_ident.to_string());
                let full_key = if let Some(ns) = &struct_config.namespace {
                    format!("{}.{}", ns, key)
                } else {
                    key
                };
                translatable_fields.push((field_ident.clone(), full_key));
            }
            None => {
                // No #[i18n] attribute — include String fields, skip others
                if is_string {
                    let key = field_ident.to_string();
                    let full_key = if let Some(ns) = &struct_config.namespace {
                        format!("{}.{}", ns, key)
                    } else {
                        key
                    };
                    translatable_fields.push((field_ident.clone(), full_key));
                }
            }
        }
    }

    if translatable_fields.is_empty() {
        return quote! {
            compile_error!("#[derive(I18n)] requires at least one String field (use #[i18n(skip)] to silence)");
        }
        .into();
    }

    let field_names: Vec<&syn::Ident> = translatable_fields.iter().map(|(f, _)| f).collect();
    let keys: Vec<&String> = translatable_fields.iter().map(|(_, k)| k).collect();
    let count = translatable_fields.len();

    let expanded = quote! {
        impl ::bevy_i18n::Localizable for #name {
            fn translations() -> &'static [(&'static str, &'static str)] {
                &[
                    #((stringify!(#field_names), #keys)),*
                ]
            }

            fn set_field(&mut self, field_name: &str, value: &str) {
                match field_name {
                    #(stringify!(#field_names) => self.#field_names = value.to_string()),*,
                    _ => {}
                }
            }
        }
    };

    TokenStream::from(expanded)
}
```

- [ ] **Step 3: Verify derive crate compiles**

```bash
cargo check -p bevy_i18n_derive
```

Expected: clean compilation.

- [ ] **Step 4: Commit**

```bash
git add derive/src/lib.rs derive/src/attr.rs
git commit -m "feat: rewrite derive macro with multi-field, namespace, and skip support"
```

---

### Task 7: Add derive feature tests

**Files:**
- Modify: `tests/i18n_test.rs` — add derive-based tests

- [ ] **Step 1: Add derive tests to tests/i18n_test.rs**

Append these tests to the end of the file:

```rust
// ── Derive macro tests ──────────────────────────────────────────────

#[test]
fn test_derive_single_field() {
    #[derive(bevy_i18n::I18n, Component)]
    struct SimpleLabel {
        text: String,
    }

    let translations = SimpleLabel::translations();
    assert_eq!(translations.len(), 1);
    assert_eq!(translations[0], ("text", "text"));
}

#[test]
fn test_derive_multi_field_with_keys() {
    #[derive(bevy_i18n::I18n, Component)]
    struct DialogBox {
        #[i18n(key = "dialog.title")]
        title: String,
        #[i18n(key = "dialog.body")]
        content: String,
        _unused: i32,
    }

    let translations = DialogBox::translations();
    assert_eq!(translations.len(), 2);
    assert_eq!(translations[0], ("title", "dialog.title"));
    assert_eq!(translations[1], ("content", "dialog.body"));
}

#[test]
fn test_derive_namespace() {
    #[derive(bevy_i18n::I18n, Component)]
    #[i18n(namespace = "hud")]
    struct HUD {
        #[i18n(key = "score")]
        score_text: String,
        #[i18n(key = "level")]
        level_text: String,
    }

    let translations = HUD::translations();
    assert_eq!(translations.len(), 2);
    assert_eq!(translations[0], ("score_text", "hud.score"));
    assert_eq!(translations[1], ("level_text", "hud.level"));
}

#[test]
fn test_derive_skip_attribute() {
    #[derive(bevy_i18n::I18n, Component)]
    struct PartialLabel {
        #[i18n(skip)]
        skipped: String,
        #[i18n(key = "label.show")]
        shown: String,
    }

    let translations = PartialLabel::translations();
    assert_eq!(translations.len(), 1);
    assert_eq!(translations[0], ("shown", "label.show"));
}

#[test]
fn test_derive_set_field() {
    #[derive(bevy_i18n::I18n, Component)]
    struct TestLabel {
        #[i18n(key = "test.label")]
        text: String,
    }

    let mut label = TestLabel { text: String::new() };
    label.set_field("text", "hello");
    assert_eq!(label.text, "hello");
}
```

- [ ] **Step 2: Run tests with derive feature**

```bash
cargo test --features derive
```

Expected: all existing tests pass + 5 new derive tests pass.

- [ ] **Step 3: Run tests without derive feature (ensure no breakage)**

```bash
cargo test
```

Expected: all existing tests pass (derive tests are gated by `#[cfg(feature = "derive")]` implicitly via the `use`).

Actually — the derive tests use `bevy_i18n::I18n` which only exists when derive is enabled. The tests need a cfg guard. Let me adjust: the test file needs `#[cfg(feature = "derive")]` on the derive test section.

The tests in Step 1 should be wrapped:

```rust
#[cfg(feature = "derive")]
mod derive_tests {
    use bevy::prelude::*;
    use bevy_i18n::prelude::*;
    use bevy_i18n::Localizable;

    #[test]
    fn test_derive_single_field() {
        #[derive(I18n, Component)]
        struct SimpleLabel {
            text: String,
        }
        let translations = SimpleLabel::translations();
        assert_eq!(translations.len(), 1);
        assert_eq!(translations[0], ("text", "text"));
    }

    #[test]
    fn test_derive_multi_field_with_keys() {
        #[derive(I18n, Component)]
        struct DialogBox {
            #[i18n(key = "dialog.title")]
            title: String,
            #[i18n(key = "dialog.body")]
            content: String,
            _unused: i32,
        }
        let translations = DialogBox::translations();
        assert_eq!(translations.len(), 2);
        assert_eq!(translations[0], ("title", "dialog.title"));
        assert_eq!(translations[1], ("content", "dialog.body"));
    }

    #[test]
    fn test_derive_namespace() {
        #[derive(I18n, Component)]
        #[i18n(namespace = "hud")]
        struct HUD {
            #[i18n(key = "score")]
            score_text: String,
            #[i18n(key = "level")]
            level_text: String,
        }
        let translations = HUD::translations();
        assert_eq!(translations.len(), 2);
        assert_eq!(translations[0], ("score_text", "hud.score"));
        assert_eq!(translations[1], ("level_text", "hud.level"));
    }

    #[test]
    fn test_derive_skip_attribute() {
        #[derive(I18n, Component)]
        struct PartialLabel {
            #[i18n(skip)]
            skipped: String,
            #[i18n(key = "label.show")]
            shown: String,
        }
        let translations = PartialLabel::translations();
        assert_eq!(translations.len(), 1);
        assert_eq!(translations[0], ("shown", "label.show"));
    }

    #[test]
    fn test_derive_set_field() {
        #[derive(I18n, Component)]
        struct TestLabel {
            #[i18n(key = "test.label")]
            text: String,
        }
        let mut label = TestLabel { text: String::new() };
        label.set_field("text", "hello");
        assert_eq!(label.text, "hello");
    }
}
```

- [ ] **Step 4: Commit**

```bash
git add tests/i18n_test.rs
git commit -m "test: add derive macro tests"
```

---

### Task 8: Update examples to work with new API

**Files:**
- Modify: `examples/basic.rs`
- Modify: `examples/locale_switch.rs`
- Modify: `examples/advanced_features.rs`

These examples use `I18nText` and `T` which are still present — they should work without changes since `update_text_system` still handles `(I18nText, Text)` pairs. The `update_localizable` and `update_i18n` functions are no longer used by examples (they're for custom components).

- [ ] **Step 1: Verify examples compile**

```bash
cargo check --examples
```

Expected: all 3 examples compile cleanly.

- [ ] **Step 2: If examples fail, fix import paths**

If any example imports `I18nKey` (which was removed), replace with the appropriate approach. Check each example:

- `basic.rs` — uses `I18nText`, no `I18nKey` → no changes
- `locale_switch.rs` — uses `I18nText`, no `I18nKey` → no changes
- `advanced_features.rs` — uses `I18nText`, no `I18nKey` → no changes

- [ ] **Step 3: Commit**

```bash
git add examples/
git commit -m "chore: verify examples compile with new API"
```

---

### Task 9: Full verification and final commit

- [ ] **Step 1: Build everything**

```bash
cargo build --all-features
```

- [ ] **Step 2: Run all tests**

```bash
cargo test --all-features
```

- [ ] **Step 3: Check examples**

```bash
cargo check --examples --features derive
```

- [ ] **Step 4: Run clippy**

```bash
cargo clippy --all-features -- -D warnings
```

- [ ] **Step 5: Final commit with all changes**

```bash
git add -A
git commit -m "feat: complete workspace restructure and derive macro overhaul"
```
