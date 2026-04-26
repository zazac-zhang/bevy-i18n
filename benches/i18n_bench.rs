use bevy::asset::Assets;
use bevy::prelude::*;
use bevy_i18n::prelude::*;
use std::collections::HashMap;

fn main() {
    divan::main();
}

fn setup_i18n() -> (bevy_i18n::prelude::I18n, Assets<I18nAsset>) {
    let mut assets = Assets::<I18nAsset>::default();
    let mut entries = HashMap::new();

    // 100 keys for realistic benchmarking
    for i in 0..100 {
        entries.insert(
            format!("key_{i}"),
            format!("Translation {i} for {{name}} with {{count}} items"),
        );
    }
    // Add plural keys
    for i in 0..10 {
        entries.insert(format!("items_{i}.zero"), format!("No items {i}"));
        entries.insert(format!("items_{i}.one"), format!("One item {i}"));
        entries.insert(format!("items_{i}.other"), format!("{{count}} items {i}"));
    }

    let handle = assets.add(I18nAsset::new(entries));
    let mut i18n = I18n::default();
    i18n.add_locale("en", handle);
    i18n.set_locale("en");

    (i18n, assets)
}

#[divan::bench]
fn lookup_static_key() {
    let (i18n, assets) = setup_i18n();
    divan::black_box(i18n.get("key_0", &[], &assets));
}

#[divan::bench]
fn lookup_with_interpolation() {
    let (i18n, assets) = setup_i18n();
    divan::black_box(i18n.get("key_50", &[("name", "World"), ("count", "42")], &assets));
}

#[divan::bench]
fn lookup_cache_hit() {
    let (i18n, assets) = setup_i18n();
    // First lookup to populate cache
    let _ = i18n.get("key_50", &[("name", "World")], &assets);
    // Second lookup hits cache
    divan::black_box(i18n.get("key_50", &[("name", "World")], &assets));
}

#[divan::bench]
fn lookup_cache_miss() {
    let (i18n, assets) = setup_i18n();
    // Different keys each time = always miss
    divan::black_box(i18n.get("nonexistent_key", &[], &assets));
}

#[divan::bench]
fn lookup_missing_key() {
    let (i18n, assets) = setup_i18n();
    divan::black_box(i18n.get("nonexistent_key", &[], &assets));
}

#[divan::bench]
fn plural_lookup() {
    let (i18n, assets) = setup_i18n();
    divan::black_box(i18n.get_plural("items_0", None, Some(5), &[], &assets));
}

#[divan::bench]
fn plural_lookup_zero() {
    let (i18n, assets) = setup_i18n();
    divan::black_box(i18n.get_plural("items_0", None, Some(0), &[], &assets));
}

#[divan::bench]
fn plural_lookup_one() {
    let (i18n, assets) = setup_i18n();
    divan::black_box(i18n.get_plural("items_0", None, Some(1), &[], &assets));
}

#[divan::bench]
fn context_lookup() {
    let (i18n, assets) = setup_i18n();
    divan::black_box(i18n.get_plural("key_0", Some("menu"), None, &[], &assets));
}

#[divan::bench]
fn locale_switch_cost() {
    let mut assets = Assets::<I18nAsset>::default();
    let mut entries = HashMap::new();
    for i in 0..100 {
        entries.insert(format!("key_{i}"), format!("Value {i}"));
    }
    let handle = assets.add(I18nAsset::new(entries));

    let mut i18n = I18n::default();
    i18n.add_locale("en", handle.clone());
    i18n.add_locale("zh", handle);
    i18n.set_locale("en");

    divan::black_box(());
    i18n.set_locale("zh");
    divan::black_box(());
}

#[divan::bench]
fn number_formatting() {
    use bevy_i18n::NumberFormat;
    let fmt = NumberFormat {
        thousands_sep: ',',
        decimal_sep: '.',
        decimal_places: Some(2),
        currency_symbol: Some("$".to_string()),
    };
    divan::black_box(fmt.format_currency("1234567.89"));
}

#[divan::bench]
fn interpolate_no_vars() {
    use bevy_i18n::interpolate::interpolate_with_format;
    divan::black_box(interpolate_with_format("Hello, world!", &[], None).into_owned());
}

#[divan::bench]
fn interpolate_with_vars() {
    use bevy_i18n::interpolate::interpolate_with_format;
    divan::black_box(
        interpolate_with_format("Hello, {name}! You have {count} messages.", &[
            ("name", "World"),
            ("count", "42"),
        ], None)
        .into_owned(),
    );
}

// ── System-level benchmarks (simulated ECS pipeline) ──────────────────

/// Benchmark: I18n.get through the full lookup path (cache + asset lookup).
#[divan::bench(args = [10, 100, 500])]
fn full_lookup_pipeline(count: usize) {
    let (i18n, assets) = setup_i18n();

    for i in 0..count {
        let key = format!("key_{}", i % 50);
        divan::black_box(i18n.get(&key, &[("name", "World")], &assets));
    }
}

/// Benchmark: I18n.get_plural through the full lookup path.
#[divan::bench(args = [10, 100, 500])]
fn full_plural_pipeline(count: usize) {
    let (i18n, assets) = setup_i18n();

    for i in 0..count {
        let idx = i % 10;
        let c = (i % 3) as u64;
        divan::black_box(i18n.get_plural(
            &format!("items_{idx}"),
            None,
            Some(c),
            &[],
            &assets,
        ));
    }
}

/// Benchmark: T component dirty-flag resolution pattern.
#[divan::bench(args = [10, 100, 500])]
fn dirty_flag_resolution(count: usize) {
    let mut t_components: Vec<T> = (0..count).map(|i| T::new(format!("key_{i}"))).collect();

    // Simulate locale change: mark all dirty
    for i in 0..count {
        t_components[i].mark_dirty();
    }

    // Simulate resolution: resolve and clear
    let (i18n, assets) = setup_i18n();
    for i in 0..count {
        let t = &mut t_components[i];
        if t.dirty {
            let _ = i18n.get_plural(&t.key, t.context.as_deref(), t.count, &[], &assets);
            t.dirty = false;
        }
    }
}

/// Benchmark: TVar change detection pattern.
#[divan::bench(args = [10, 100])]
fn tvar_change_detection(count: usize) {
    let tvar_values: Vec<TVar> = (0..count).map(|i| TVar::new(format!("{i}"))).collect();
    let mut t_components: Vec<(T, usize)> = (0..count)
        .map(|i| (T::new("player.score").with_dynamic_var("score", Entity::from_raw_u32(0).unwrap()), i % count))
        .collect();

    // Check for changes (simulating tvar lookup)
    for i in 0..count {
        let tvar_idx = t_components[i].1;
        let new_value = tvar_values[tvar_idx].value.clone();
        let old_value = t_components[i].0
            .vars
            .iter()
            .find(|(k, _)| k == "score")
            .map(|(_, v)| v.clone());

        if old_value.as_deref() != Some(&new_value) {
            t_components[i].0.mark_dirty();
        }
    }
}

/// Benchmark: cache hit rate under load.
#[divan::bench(args = [10, 100, 500])]
fn cache_hit_rate(count: usize) {
    let (i18n, assets) = setup_i18n();

    // Warm up cache with repeated keys
    for _ in 0..count {
        divan::black_box(i18n.get("key_0", &[("name", "World")], &assets));
    }
}

/// Benchmark: many different keys (cache miss pressure).
#[divan::bench(args = [10, 100, 500])]
fn cache_pressure(count: usize) {
    let (i18n, assets) = setup_i18n();

    for i in 0..count {
        divan::black_box(i18n.get(&format!("key_{i}"), &[("name", "World")], &assets));
    }
}
