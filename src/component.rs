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
impl Localizable for Text {
    fn translations() -> &'static [(&'static str, &'static str)] {
        &[("0", "_text_")]
    }
    fn set_field(&mut self, _field_name: &str, value: &str) {
        self.0 = value.to_string();
    }
}

/// Component that marks an entity for automatic i18n translation.
///
/// When `key` is set, only the matching field in the paired `Localizable` component is translated.
/// When `key` is `None`, all fields from `Localizable::translations()` are translated.
///
/// Add this alongside Bevy's `Text` component or any custom `Localizable` component
/// to have content automatically resolved from the current locale.
///
/// # Example
/// ```ignore
/// // Text translation
/// commands.spawn((
///     Text::new(""),
///     I18nMarker::new("game.title"),
/// ));
///
/// // Custom component (all fields translated)
/// commands.spawn((
///     I18nMarker::marker(),
///     DialogBox { title: String::new(), body: String::new() },
/// ));
/// ```
#[derive(Component, Clone, Debug)]
pub struct I18nMarker {
    /// Translation key (e.g. "game.title"). None = translate all fields via Localizable::translations().
    pub key: Option<String>,
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

impl I18nMarker {
    /// Create a marker for a specific translation key.
    ///
    /// Only the field whose key matches will be translated.
    /// This is the constructor used for `Text` and single-field components.
    pub fn new(key: impl Into<String>) -> Self {
        Self {
            key: Some(key.into()),
            context: None,
            vars: Vec::new(),
            dynamic_vars: Vec::new(),
            count: None,
            dirty: true,
        }
    }

    /// Create an empty marker. All fields from the paired `Localizable` component
    /// will be translated automatically.
    pub fn marker() -> Self {
        Self {
            key: None,
            context: None,
            vars: Vec::new(),
            dynamic_vars: Vec::new(),
            count: None,
            dirty: true,
        }
    }

    /// Create a marker with a context for disambiguation.
    ///
    /// The context is prepended to the key as `context::key`.
    /// This is useful when the same word has different translations
    /// in different contexts (e.g. "file" as noun vs verb).
    ///
    /// # Example
    /// ```ignore
    /// I18nMarker::with_context("open", "menu")     // looks up "menu::open"
    /// I18nMarker::with_context("open", "dialog")   // looks up "dialog::open"
    /// ```
    pub fn with_context(key: impl Into<String>, context: impl Into<String>) -> Self {
        Self {
            key: Some(key.into()),
            context: Some(context.into()),
            vars: Vec::new(),
            dynamic_vars: Vec::new(),
            count: None,
            dirty: true,
        }
    }

    /// Create a marker with variable substitutions.
    ///
    /// # Example
    /// ```ignore
    /// I18nMarker::with_vars("player.greeting", &[("name", "Hero")])
    /// ```
    pub fn with_vars(key: impl Into<String>, vars: &[(&str, &str)]) -> Self {
        Self {
            key: Some(key.into()),
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

    /// Create a marker for a plural translation key.
    ///
    /// The `count` parameter determines which plural form to use:
    /// - `0` → `key.zero` (or `key.other` if zero is missing)
    /// - `1` → `key.one`
    /// - `2+` → `key.other`
    ///
    /// # Example
    /// ```ignore
    /// I18nMarker::plural("player.inventory", items.len() as u64)
    /// ```
    pub fn plural(key: impl Into<String>, count: u64) -> Self {
        Self {
            key: Some(key.into()),
            context: None,
            vars: Vec::new(),
            dynamic_vars: Vec::new(),
            count: Some(count),
            dirty: true,
        }
    }

    /// Create a marker with a dynamic variable reference.
    ///
    /// The `tvar_entity` should have a `TVar` component. The variable value
    /// is read fresh each frame from that entity.
    ///
    /// # Example
    /// ```ignore
    /// // Spawn a dynamic variable
    /// let score_entity = commands.spawn(TVar::new("0")).id();
    ///
    /// // Reference it in an I18nMarker component
    /// commands.spawn((
    ///     Text::new(""),
    ///     I18nMarker::new("player.score")
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
    /// I18nMarker::ns("ui.menu").key("quit")        // looks up "ui.menu.quit"
    /// I18nMarker::ns("settings.audio").key("volume") // looks up "settings.audio.volume"
    /// ```
    pub fn ns(namespace: impl Into<String>) -> NamespaceBuilder {
        NamespaceBuilder {
            namespace: namespace.into(),
        }
    }
}

/// Component that stores a dynamic variable value.
///
/// Spawn this on an entity and reference it from `I18nMarker::with_dynamic_var`.
/// When the TVar value changes, associated I18nMarker components will update automatically.
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

/// Builder for namespaced I18nMarker components. Created by `I18nMarker::ns()`.
#[derive(Clone, Debug)]
pub struct NamespaceBuilder {
    namespace: String,
}

impl NamespaceBuilder {
    /// Create an I18nMarker component with the namespaced key.
    pub fn key(self, key: impl Into<String>) -> I18nMarker {
        let key: String = key.into();
        I18nMarker::new(format!("{}.{}", self.namespace, key))
    }

    /// Create an I18nMarker component with namespaced key and variable substitutions.
    pub fn with_vars(self, key: impl Into<String>, vars: &[(&str, &str)]) -> I18nMarker {
        let key: String = key.into();
        I18nMarker::with_vars(format!("{}.{}", self.namespace, key), vars)
    }

    /// Create an I18nMarker component with namespaced key and plural count.
    pub fn plural(self, key: impl Into<String>, count: u64) -> I18nMarker {
        let key: String = key.into();
        I18nMarker::plural(format!("{}.{}", self.namespace, key), count)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_i18n_marker_new() {
        let t = I18nMarker::new("game.title");
        assert_eq!(t.key, Some("game.title".to_string()));
        assert!(t.vars.is_empty());
        assert!(t.count.is_none());
        assert!(t.dirty);
    }

    #[test]
    fn test_i18n_marker_empty_marker() {
        let t = I18nMarker::marker();
        assert!(t.key.is_none());
        assert!(t.vars.is_empty());
        assert!(t.count.is_none());
        assert!(t.dirty);
    }

    #[test]
    fn test_i18n_marker_with_vars() {
        let t = I18nMarker::with_vars("greeting", &[("name", "World")]);
        assert_eq!(t.key, Some("greeting".to_string()));
        assert_eq!(t.vars.len(), 1);
        assert!(t.count.is_none());
        assert!(t.dirty);
    }

    #[test]
    fn test_i18n_marker_plural() {
        let t = I18nMarker::plural("items.count", 5);
        assert_eq!(t.key, Some("items.count".to_string()));
        assert_eq!(t.count, Some(5));
        assert!(t.dirty);
    }

    #[test]
    fn test_i18n_marker_mark_dirty() {
        let mut t = I18nMarker::new("key");
        t.dirty = false;
        t.mark_dirty();
        assert!(t.dirty);
    }

    #[test]
    fn test_ns_builder() {
        let t = I18nMarker::ns("ui.menu").key("quit");
        assert_eq!(t.key, Some("ui.menu.quit".to_string()));
        assert!(t.context.is_none());
        assert!(t.vars.is_empty());
        assert!(t.dirty);
    }

    #[test]
    fn test_ns_with_vars() {
        let t = I18nMarker::ns("player").with_vars("greeting", &[("name", "Hero")]);
        assert_eq!(t.key, Some("player.greeting".to_string()));
        assert_eq!(t.vars.len(), 1);
    }

    #[test]
    fn test_ns_plural() {
        let t = I18nMarker::ns("inventory").plural("items", 5);
        assert_eq!(t.key, Some("inventory.items".to_string()));
        assert_eq!(t.count, Some(5));
    }

    #[test]
    fn test_tvar_new() {
        let tvar = TVar::new("42");
        assert_eq!(tvar.value, "42");
    }

    #[test]
    fn test_i18n_marker_with_dynamic_var() {
        let tvar_entity = Entity::from_raw_u32(0).unwrap();
        let t = I18nMarker::new("player.score").with_dynamic_var("score", tvar_entity);
        assert_eq!(t.key, Some("player.score".to_string()));
        assert_eq!(t.dynamic_vars.len(), 1);
        assert_eq!(t.dynamic_vars[0], ("score".to_string(), tvar_entity));
    }
}
