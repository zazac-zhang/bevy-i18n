use bevy::prelude::{Component, Entity, Text};

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

/// Component that marks a Text entity for automatic i18n translation.
///
/// Add this alongside Bevy's `Text` component to have the text
/// content automatically resolved from the current locale.
///
/// # Naming
///
/// - **Preferred**: `I18nText` - explicit, avoids conflicts with potential
///   Bevy `Translation` components or other libraries
/// - **Alias**: `T` - for brevity in frequently-used code
///
/// # Why I18nText?
///
/// 1. **Avoids naming conflicts**: Won't clash with Bevy's `Transform.translation`
///    or potential future `Translation` components
/// 2. **Clear context**: Immediately indicates internationalization
/// 3. **Searchable**: Unique name easy to find in codebases
///
/// # Example
/// ```ignore
/// // Preferred: explicit and clear
/// commands.spawn((
///     Text::new(""),
///     I18nText::new("game.title"),
/// ));
///
/// // Also supported: brief and familiar
/// commands.spawn((
///     Text::new(""),
///     T::new("game.title"),
/// ));
/// ```
#[derive(Component, Clone, Debug)]
pub struct I18nText {
    /// Translation key (e.g. "game.title")
    pub key: String,
    /// Optional context for disambiguation (e.g. "menu", "dialog")
    pub context: Option<String>,
    /// Variable substitutions (key -> value)
    pub vars: Vec<(String, String)>,
    /// Dynamic variable references: (var_name, Entity with TVar)
    pub dynamic_vars: Vec<(String, Entity)>,
    /// Count for plural form selection (None = static)
    pub count: Option<u64>,
    /// Whether the text needs to be re-resolved
    pub dirty: bool,
}

impl I18nText {
    /// Create an i18n text component for a static translation key.
    pub fn new(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            context: None,
            vars: Vec::new(),
            dynamic_vars: Vec::new(),
            count: None,
            dirty: true,
        }
    }

    /// Create an i18n text component with a context for disambiguation.
    ///
    /// The context is prepended to the key as `context::key`.
    /// This is useful when the same word has different translations
    /// in different contexts (e.g. "file" as noun vs verb).
    ///
    /// # Example
    /// ```ignore
    /// I18nText::with_context("open", "menu")     // looks up "menu::open"
    /// I18nText::with_context("open", "dialog")   // looks up "dialog::open"
    /// ```
    pub fn with_context(key: impl Into<String>, context: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            context: Some(context.into()),
            vars: Vec::new(),
            dynamic_vars: Vec::new(),
            count: None,
            dirty: true,
        }
    }

    /// Create an i18n text component with variable substitutions.
    ///
    /// # Example
    /// ```ignore
    /// I18nText::with_vars("player.greeting", &[("name", "Hero")])
    /// ```
    pub fn with_vars(key: impl Into<String>, vars: &[(&str, &str)]) -> Self {
        Self {
            key: key.into(),
            context: None,
            vars: vars
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            dynamic_vars: Vec::new(),
            count: None,
            dirty: true,
        }
    }

    /// Create an i18n text component for a plural translation key.
    ///
    /// The `count` parameter determines which plural form to use:
    /// - `0` → `key.zero` (or `key.other` if zero is missing)
    /// - `1` → `key.one`
    /// - `2+` → `key.other`
    ///
    /// # Example
    /// ```ignore
    /// I18nText::plural("player.inventory", items.len() as u64)
    /// ```
    pub fn plural(key: impl Into<String>, count: u64) -> Self {
        Self {
            key: key.into(),
            context: None,
            vars: Vec::new(),
            dynamic_vars: Vec::new(),
            count: Some(count),
            dirty: true,
        }
    }

    /// Create an i18n text component with a dynamic variable reference.
    ///
    /// The `tvar_entity` should have a `TVar` component. The variable value
    /// is read fresh each frame from that entity.
    ///
    /// # Example
    /// ```ignore
    /// // Spawn a dynamic variable
    /// let score_entity = commands.spawn(TVar::new("0")).id();
    ///
    /// // Reference it in an I18nText component
    /// commands.spawn((
    ///     Text::new(""),
    ///     I18nText::new("player.score")
    ///         .with_dynamic_var("score", score_entity),
    /// ));
    /// ```
    pub fn with_dynamic_var(mut self, var_name: impl Into<String>, tvar_entity: Entity) -> Self {
        self.dynamic_vars.push((var_name.into(), tvar_entity));
        self
    }

    /// Mark this component as needing re-resolution.
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Create a namespace builder for namespaced key lookup.
    ///
    /// Use this to organize translations by module in large projects.
    /// The namespace is prepended to the key as `namespace.key`.
    ///
    /// # Example
    /// ```ignore
    /// I18nText::ns("ui.menu").key("quit")        // looks up "ui.menu.quit"
    /// I18nText::ns("settings.audio").key("volume") // looks up "settings.audio.volume"
    /// ```
    pub fn ns(namespace: impl Into<String>) -> NamespaceBuilder {
        NamespaceBuilder {
            namespace: namespace.into(),
        }
    }
}

/// Component that stores a dynamic variable value.
///
/// Spawn this on an entity and reference it from `I18nText::with_dynamic_var`.
/// When the TVar value changes, associated I18nText components will update automatically.
///
/// # Example
/// ```ignore
/// let score = commands.spawn(TVar::new("0")).id();
///
/// // Later, update the score
/// commands.entity(score).insert(TVar::new("100"));
/// ```
#[derive(Component, Clone, Debug, Default)]
pub struct TVar {
    /// The current value of this variable.
    pub value: String,
}

impl TVar {
    /// Create a TVar with the given value.
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
        }
    }
}

/// Builder for namespaced I18nText components. Created by `I18nText::ns()`.
#[derive(Clone, Debug)]
pub struct NamespaceBuilder {
    namespace: String,
}

impl NamespaceBuilder {
    /// Create an I18nText component with the namespaced key.
    pub fn key(self, key: impl Into<String>) -> I18nText {
        let key: String = key.into();
        I18nText::new(format!("{}.{}", self.namespace, key))
    }

    /// Create an I18nText component with namespaced key and variable substitutions.
    pub fn with_vars(self, key: impl Into<String>, vars: &[(&str, &str)]) -> I18nText {
        let key: String = key.into();
        I18nText::with_vars(format!("{}.{}", self.namespace, key), vars)
    }

    /// Create an I18nText component with namespaced key and plural count.
    pub fn plural(self, key: impl Into<String>, count: u64) -> I18nText {
        let key: String = key.into();
        I18nText::plural(format!("{}.{}", self.namespace, key), count)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_i18n_text_new() {
        let t = I18nText::new("game.title");
        assert_eq!(t.key, "game.title");
        assert!(t.vars.is_empty());
        assert!(t.count.is_none());
        assert!(t.dirty);
    }

    #[test]
    fn test_i18n_text_with_vars() {
        let t = I18nText::with_vars("greeting", &[("name", "World")]);
        assert_eq!(t.key, "greeting");
        assert_eq!(t.vars.len(), 1);
        assert!(t.count.is_none());
        assert!(t.dirty);
    }

    #[test]
    fn test_i18n_text_plural() {
        let t = I18nText::plural("items.count", 5);
        assert_eq!(t.key, "items.count");
        assert_eq!(t.count, Some(5));
        assert!(t.dirty);
    }

    #[test]
    fn test_i18n_text_mark_dirty() {
        let mut t = I18nText::new("key");
        t.dirty = false;
        t.mark_dirty();
        assert!(t.dirty);
    }

    #[test]
    fn test_ns_builder() {
        let t = I18nText::ns("ui.menu").key("quit");
        assert_eq!(t.key, "ui.menu.quit");
        assert!(t.context.is_none());
        assert!(t.vars.is_empty());
        assert!(t.dirty);
    }

    #[test]
    fn test_ns_with_vars() {
        let t = I18nText::ns("player").with_vars("greeting", &[("name", "Hero")]);
        assert_eq!(t.key, "player.greeting");
        assert_eq!(t.vars.len(), 1);
    }

    #[test]
    fn test_ns_plural() {
        let t = I18nText::ns("inventory").plural("items", 5);
        assert_eq!(t.key, "inventory.items");
        assert_eq!(t.count, Some(5));
    }

    #[test]
    fn test_tvar_new() {
        let tvar = TVar::new("42");
        assert_eq!(tvar.value, "42");
    }

    #[test]
    fn test_i18n_text_with_dynamic_var() {
        let tvar_entity = Entity::from_raw_u32(0).unwrap();
        let t = I18nText::new("player.score").with_dynamic_var("score", tvar_entity);
        assert_eq!(t.key, "player.score");
        assert_eq!(t.dynamic_vars.len(), 1);
        assert_eq!(t.dynamic_vars[0], ("score".to_string(), tvar_entity));
    }
}
