# Bevy I18n 目录重构与派生宏拓展设计

## 概述

整合混乱的 `derive/` 和 `bevy_i18n_macro/` 目录，建立 workspace 结构。统一 `Localizable` trait 设计，不再区分单字段/多字段。

## 1. 目录结构

```
bevy_I18n/
├── Cargo.toml              # workspace = { members = [".", "derive"] }
├── src/                    # bevy_i18n 主 crate
│   ├── lib.rs              # 重新导出 derive feature + prelude
│   ├── asset.rs            # I18nAsset
│   ├── component.rs        # Localizable trait (统一设计)
│   ├── systems.rs          # update_localizable<T>
│   ├── interpolate.rs
│   ├── plugin.rs
│   ├── resource.rs
│   └── bin/                # CLI 工具
├── derive/                 # bevy_i18n_derive (proc-macro)
│   ├── Cargo.toml
│   └── src/lib.rs
├── examples/               # basic, locale_switch, advanced_features
├── tests/
├── benches/
├── assets/locales/
└── docs/superpowers/
```

**删除**：`bevy_i18n_macro/`（空壳）。

## 2. Workspace 配置

根 `Cargo.toml` 添加：

```toml
[workspace]
members = [".", "derive"]

[features]
derive = ["dep:bevy_i18n_derive"]
```

derive 作为 optional 依赖，用户通过 `features = ["derive"]` 启用。

## 3. Localizable Trait（统一设计）

删除旧的 `Localizable`，替换为统一版本。不再区分单字段/多字段：

```rust
pub trait Localizable: Component {
    /// [(field_name, translation_key), ...]
    fn translations() -> &'static [(&'static str, &'static str)];
    fn set_field(&mut self, field_name: &str, value: &str);
}
```

单字段结构体也生成一个元素的数组。

## 4. 派生宏属性

| 属性 | 行为 |
|------|------|
| `#[i18n(key = "...")]` | 指定翻译 key |
| `#[i18n]` | 字段名作为 key |
| `#[i18n(skip)]` | 显式跳过 |

结构体级别：`#[i18n(namespace = "...")]` 自动为所有字段 key 添加前缀。

非 String 字段自动忽略。

## 5. 生成代码示例

```rust
#[derive(I18n, Component)]
#[i18n(namespace = "dialog")]
struct DialogBox {
    #[i18n(key = "title")]
    title: String,
    #[i18n(skip)]
    temp_cache: String,
    color: Color,
}

// 生成：
impl Localizable for DialogBox {
    fn translations() -> &'static [(&'static str, &'static str)] {
        &[("title", "dialog.title")]
    }
    fn set_field(&mut self, field: &str, value: &str) {
        match field {
            "title" => self.title = value.into(),
            _ => {}
        }
    }
}
```

## 6. 更新系统

删除旧的 `update_text_system`，统一使用泛型系统：

```rust
pub fn update_localizable<T: Localizable + Component<Mutability = Mutable>>(
    i18n: Res<I18n>,
    locales: Res<Assets<I18nAsset>>,
    mut query: Query<&mut T>,
) {
    for mut component in &mut query {
        for (field, key) in T::translations() {
            let translated = i18n.get(key, &[], &locales);
            component.set_field(field, &translated);
        }
    }
}
```

## 7. Text 组件适配

`Text` 也实现 `Localizable`，保持原有使用方式：

```rust
impl Localizable for (I18nKey, Text) {
    fn translations() -> &'static [(&'static str, &'static str)] {
        &[("text", "_key_placeholder_")] // 由 I18nKey.key 决定实际 key
    }
    // ...
}
```

实际上 `(I18nKey, Text)` 组合仍然由现有 `update_text_system` 处理，derive 系统用于自定义组件。两者共存。

## 8. prelude 导出

当 `derive` feature 启用时，prelude 中导出 `I18n` derive macro 和 `Localizable` trait。
