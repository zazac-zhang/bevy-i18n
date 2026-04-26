use bevy::prelude::Component;

/// Component that marks a Text entity for automatic translation.
///
/// Add this alongside Bevy's `Text` component to have the text
/// content automatically resolved from the current locale.
///
/// # Example
/// ```ignore
/// commands.spawn((
///     Text::new(""),
///     T::new("game.title"),
/// ));
/// ```
#[derive(Component, Clone, Debug)]
pub struct T {
    /// Translation key (e.g. "game.title")
    pub key: String,
    /// Optional context for disambiguation (e.g. "menu", "dialog")
    pub context: Option<String>,
    /// Variable substitutions (key -> value)
    pub vars: Vec<(String, String)>,
    /// Count for plural form selection (None = static)
    pub count: Option<u64>,
    /// Whether the text needs to be re-resolved
    pub dirty: bool,
}

impl T {
    /// Create a T component for a static translation key.
    pub fn new(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            context: None,
            vars: Vec::new(),
            count: None,
            dirty: true,
        }
    }

    /// Create a T component with a context for disambiguation.
    ///
    /// The context is prepended to the key as `context::key`.
    /// This is useful when the same word has different translations
    /// in different contexts (e.g. "file" as noun vs verb).
    ///
    /// # Example
    /// ```ignore
    /// T::with_context("open", "menu")     // looks up "menu::open"
    /// T::with_context("open", "dialog")   // looks up "dialog::open"
    /// ```
    pub fn with_context(key: impl Into<String>, context: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            context: Some(context.into()),
            vars: Vec::new(),
            count: None,
            dirty: true,
        }
    }

    /// Create a T component with variable substitutions.
    ///
    /// # Example
    /// ```ignore
    /// T::with_vars("player.greeting", &[("name", "Hero")])
    /// ```
    pub fn with_vars(key: impl Into<String>, vars: &[(&str, &str)]) -> Self {
        Self {
            key: key.into(),
            context: None,
            vars: vars
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            count: None,
            dirty: true,
        }
    }

    /// Create a T component for a plural translation key.
    ///
    /// The `count` parameter determines which plural form to use:
    /// - `0` → `key.zero` (or `key.other` if zero is missing)
    /// - `1` → `key.one`
    /// - `2+` → `key.other`
    ///
    /// # Example
    /// ```ignore
    /// T::plural("player.inventory", items.len() as u64)
    /// ```
    pub fn plural(key: impl Into<String>, count: u64) -> Self {
        Self {
            key: key.into(),
            context: None,
            vars: Vec::new(),
            count: Some(count),
            dirty: true,
        }
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
    /// T::ns("ui.menu").key("quit")        // looks up "ui.menu.quit"
    /// T::ns("settings.audio").key("volume") // looks up "settings.audio.volume"
    /// ```
    pub fn ns(namespace: impl Into<String>) -> NamespaceBuilder {
        NamespaceBuilder {
            namespace: namespace.into(),
        }
    }
}

/// Builder for namespaced T components. Created by `T::ns()`.
#[derive(Clone, Debug)]
pub struct NamespaceBuilder {
    namespace: String,
}

impl NamespaceBuilder {
    /// Create a T component with the namespaced key.
    pub fn key(self, key: impl Into<String>) -> T {
        let key: String = key.into();
        T::new(format!("{}.{}", self.namespace, key))
    }

    /// Create a T component with namespaced key and variable substitutions.
    pub fn with_vars(self, key: impl Into<String>, vars: &[(&str, &str)]) -> T {
        let key: String = key.into();
        T::with_vars(format!("{}.{}", self.namespace, key), vars)
    }

    /// Create a T component with namespaced key and plural count.
    pub fn plural(self, key: impl Into<String>, count: u64) -> T {
        let key: String = key.into();
        T::plural(format!("{}.{}", self.namespace, key), count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_t_new() {
        let t = T::new("game.title");
        assert_eq!(t.key, "game.title");
        assert!(t.vars.is_empty());
        assert!(t.count.is_none());
        assert!(t.dirty);
    }

    #[test]
    fn test_t_with_vars() {
        let t = T::with_vars("greeting", &[("name", "World")]);
        assert_eq!(t.key, "greeting");
        assert_eq!(t.vars.len(), 1);
        assert!(t.count.is_none());
        assert!(t.dirty);
    }

    #[test]
    fn test_t_plural() {
        let t = T::plural("items.count", 5);
        assert_eq!(t.key, "items.count");
        assert_eq!(t.count, Some(5));
        assert!(t.dirty);
    }

    #[test]
    fn test_t_mark_dirty() {
        let mut t = T::new("key");
        t.dirty = false;
        t.mark_dirty();
        assert!(t.dirty);
    }

    #[test]
    fn test_ns_builder() {
        let t = T::ns("ui.menu").key("quit");
        assert_eq!(t.key, "ui.menu.quit");
        assert!(t.context.is_none());
        assert!(t.vars.is_empty());
        assert!(t.dirty);
    }

    #[test]
    fn test_ns_with_vars() {
        let t = T::ns("player").with_vars("greeting", &[("name", "Hero")]);
        assert_eq!(t.key, "player.greeting");
        assert_eq!(t.vars.len(), 1);
    }

    #[test]
    fn test_ns_plural() {
        let t = T::ns("inventory").plural("items", 5);
        assert_eq!(t.key, "inventory.items");
        assert_eq!(t.count, Some(5));
    }
}
