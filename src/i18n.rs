//! Internationalization (i18n) framework for Dewey.
//!
//! Provides message catalogs and locale-based text lookup so widgets
//! and applications can display translated strings.

use std::collections::HashMap;

use crate::ontology::*;

/// A locale identifier (e.g. "en", "en-US", "ja").
pub type Locale = String;

/// A message catalog containing translations for a single locale.
#[derive(Debug, Clone, Default)]
pub struct MessageCatalog {
    messages: HashMap<String, String>,
}

impl MessageCatalog {
    /// Create an empty catalog.
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a message translation.
    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.messages.insert(key.into(), value.into());
    }

    /// Builder: add a message.
    pub fn with(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.insert(key, value);
        self
    }

    /// Look up a message by key.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.messages.get(key).map(|s| s.as_str())
    }

    /// Load messages from a JSON string: `{ "key": "value", ... }`.
    pub fn from_json(json: &str) -> Result<Self, String> {
        let map: HashMap<String, String> =
            serde_json::from_str(json).map_err(|e| e.to_string())?;
        Ok(Self { messages: map })
    }
}

/// The i18n manager. Holds catalogs for multiple locales and resolves
/// translations against the active locale.
#[derive(Debug, Clone)]
pub struct I18n {
    catalogs: HashMap<Locale, MessageCatalog>,
    active_locale: Locale,
    fallback_locale: Locale,
}

impl I18n {
    /// Create a new manager with a default locale.
    pub fn new(default_locale: impl Into<Locale>) -> Self {
        let locale = default_locale.into();
        Self {
            catalogs: HashMap::new(),
            active_locale: locale.clone(),
            fallback_locale: locale,
        }
    }

    /// Register a catalog for a locale.
    pub fn add_catalog(&mut self, locale: impl Into<Locale>, catalog: MessageCatalog) {
        self.catalogs.insert(locale.into(), catalog);
    }

    /// Set the active locale.
    pub fn set_locale(&mut self, locale: impl Into<Locale>) {
        self.active_locale = locale.into();
    }

    /// Get the active locale.
    pub fn locale(&self) -> &str {
        &self.active_locale
    }

    /// Translate a message key. Falls back to the fallback locale, then
    /// returns the key itself if no translation is found.
    pub fn t<'a>(&'a self, key: &'a str) -> &'a str {
        if let Some(catalog) = self.catalogs.get(&self.active_locale) {
            if let Some(msg) = catalog.get(key) {
                return msg;
            }
        }
        if self.active_locale != self.fallback_locale {
            if let Some(catalog) = self.catalogs.get(&self.fallback_locale) {
                if let Some(msg) = catalog.get(key) {
                    return msg;
                }
            }
        }
        key
    }

    /// Translate with parameter substitution. Replaces `{0}`, `{1}`, etc.
    pub fn t_fmt(&self, key: &str, args: &[&str]) -> String {
        let base = self.t(key).to_string();
        let mut result = base;
        for (i, arg) in args.iter().enumerate() {
            result = result.replace(&format!("{{{i}}}"), arg);
        }
        result
    }

    /// Get all registered locales.
    pub fn locales(&self) -> Vec<&str> {
        self.catalogs.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for I18n {
    fn default() -> Self {
        Self::new("en")
    }
}

impl Discoverable for I18n {
    fn schema(&self) -> WidgetSchema {
        let mut schema = WidgetSchema::new(
            "I18n",
            "Internationalization manager with locale-based translations and fallback",
            SemanticRole::System,
        );
        schema.usage_hint = Some("i18n.t(\"hello\") or i18n.t_fmt(\"greet\", &[\"world\"])".into());
        schema.tags = vec!["i18n".into(), "locale".into(), "translation".into(), "language".into()];
        schema
    }

    fn capabilities(&self) -> Vec<AgentCapability> {
        vec![]
    }

    fn actions(&self) -> Vec<AgentAction> {
        vec![
            AgentAction::with_params(
                "set_locale",
                "Switch the active locale",
                vec![ActionParam::required("locale", "Locale identifier (e.g. en, es, ja)", ActionParamType::String)],
                true,
            ),
            AgentAction::with_params(
                "translate",
                "Look up a translation key",
                vec![ActionParam::required("key", "Message key", ActionParamType::String)],
                false,
            ),
            AgentAction::simple("list_locales", "List all available locales", false),
        ]
    }

    fn semantic_role(&self) -> SemanticRole {
        SemanticRole::System
    }

    fn agent_state(&self) -> serde_json::Value {
        serde_json::json!({
            "active_locale": self.active_locale,
            "fallback_locale": self.fallback_locale,
            "available_locales": self.locales(),
            "catalog_count": self.catalogs.len(),
        })
    }

    fn execute_action(
        &mut self,
        action: &str,
        params: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        match action {
            "set_locale" => {
                let locale = params["locale"]
                    .as_str()
                    .ok_or("missing locale")?
                    .to_string();
                self.set_locale(locale.clone());
                Ok(serde_json::json!({ "active_locale": locale }))
            }
            "translate" => {
                let key = params["key"].as_str().ok_or("missing key")?;
                let result = self.t(key).to_string();
                Ok(serde_json::json!({ "key": key, "translation": result }))
            }
            "list_locales" => {
                Ok(serde_json::json!({ "locales": self.locales() }))
            }
            _ => Err(format!("Unknown action: {action}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_translation() {
        let mut i18n = I18n::new("en");
        i18n.add_catalog(
            "en",
            MessageCatalog::new()
                .with("hello", "Hello")
                .with("bye", "Goodbye"),
        );
        i18n.add_catalog("es", MessageCatalog::new().with("hello", "Hola"));
        assert_eq!(i18n.t("hello"), "Hello");
        i18n.set_locale("es");
        assert_eq!(i18n.t("hello"), "Hola");
        // Fallback to "en" for missing key
        assert_eq!(i18n.t("bye"), "Goodbye");
    }

    #[test]
    fn parameter_substitution() {
        let mut i18n = I18n::new("en");
        i18n.add_catalog(
            "en",
            MessageCatalog::new().with("greet", "Hello, {0}! You have {1} items."),
        );
        assert_eq!(
            i18n.t_fmt("greet", &["Alice", "5"]),
            "Hello, Alice! You have 5 items."
        );
    }

    #[test]
    fn missing_key_returns_key() {
        let i18n = I18n::new("en");
        assert_eq!(i18n.t("nonexistent"), "nonexistent");
    }

    #[test]
    fn from_json() {
        let json = r#"{"hello": "Hello", "world": "World"}"#;
        let catalog = MessageCatalog::from_json(json).unwrap();
        assert_eq!(catalog.get("hello"), Some("Hello"));
        assert_eq!(catalog.get("world"), Some("World"));
    }
}
