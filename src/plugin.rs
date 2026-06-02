//! Plugin system for Dewey.
//!
//! Plugins allow third-party code to register custom widgets, themes,
//! message catalogs, and agent capabilities at application startup.

use crate::i18n::I18n;
use crate::ontology::*;
use crate::theme::Theme;

/// Host context passed to plugins during initialization.
///
/// Plugins use this to register their contributions.
pub struct PluginContext<'a> {
    /// The ontology registry.
    pub ontology: &'a mut OntologyRegistry,
    /// The i18n manager.
    pub i18n: &'a mut I18n,
    /// A mutable reference to the active theme, allowing plugins to
    /// extend or override tokens.
    pub theme: &'a mut Theme,
}

/// Trait that all Dewey plugins must implement.
pub trait Plugin: Send {
    /// Human-readable plugin name.
    fn name(&self) -> &str;

    /// Semantic version string.
    fn version(&self) -> &str {
        "0.1.0"
    }

    /// Called once at application startup.
    fn init(&mut self, ctx: &mut PluginContext<'_>);

    /// Called each frame before rendering. Optional hook.
    fn on_frame(&mut self) {}

    /// Called when the application is shutting down. Optional hook.
    fn on_shutdown(&mut self) {}
}

/// A registry that manages plugins.
pub struct PluginRegistry {
    plugins: Vec<Box<dyn Plugin>>,
}

impl PluginRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }

    /// Register a plugin.
    pub fn register(&mut self, plugin: impl Plugin + 'static) {
        self.plugins.push(Box::new(plugin));
    }

    /// Initialize all registered plugins.
    pub fn init_all(&mut self, ctx: &mut PluginContext<'_>) {
        for plugin in &mut self.plugins {
            log::info!(
                "Initializing plugin: {} v{}",
                plugin.name(),
                plugin.version()
            );
            plugin.init(ctx);
        }
    }

    /// Call the per-frame hook on all plugins.
    pub fn on_frame(&mut self) {
        for plugin in &mut self.plugins {
            plugin.on_frame();
        }
    }

    /// Call the shutdown hook on all plugins.
    pub fn on_shutdown(&mut self) {
        for plugin in &mut self.plugins {
            plugin.on_shutdown();
        }
    }

    /// Get the number of registered plugins.
    pub fn len(&self) -> usize {
        self.plugins.len()
    }

    /// Check if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.plugins.is_empty()
    }

    /// Get the names and versions of all registered plugins.
    pub fn list(&self) -> Vec<(&str, &str)> {
        self.plugins
            .iter()
            .map(|p| (p.name(), p.version()))
            .collect()
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl Discoverable for PluginRegistry {
    fn schema(&self) -> WidgetSchema {
        let mut schema = WidgetSchema::new(
            "PluginRegistry",
            "Manages third-party plugins that extend the framework",
            SemanticRole::System,
        );
        schema.usage_hint =
            Some("registry.register(MyPlugin); registry.init_all(&mut ctx);".into());
        schema.tags = vec!["plugin".into(), "extension".into(), "registry".into()];
        schema
    }

    fn capabilities(&self) -> Vec<AgentCapability> {
        vec![]
    }

    fn actions(&self) -> Vec<AgentAction> {
        vec![AgentAction::simple(
            "list",
            "List all registered plugins",
            false,
        )]
    }

    fn semantic_role(&self) -> SemanticRole {
        SemanticRole::System
    }

    fn agent_state(&self) -> serde_json::Value {
        let plugins: Vec<serde_json::Value> = self
            .list()
            .iter()
            .map(|(name, version)| serde_json::json!({ "name": name, "version": version }))
            .collect();
        serde_json::json!({
            "plugin_count": self.plugins.len(),
            "plugins": plugins,
        })
    }

    fn execute_action(
        &mut self,
        action: &str,
        _params: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        match action {
            "list" => Ok(self.agent_state()),
            _ => Err(format!("Unknown action: {action}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::I18n;
    use crate::ontology::OntologyRegistry;
    use crate::theme::Theme;

    struct TestPlugin {
        init_called: bool,
    }

    impl TestPlugin {
        fn new() -> Self {
            Self { init_called: false }
        }
    }

    impl Plugin for TestPlugin {
        fn name(&self) -> &str {
            "test-plugin"
        }

        fn version(&self) -> &str {
            "1.0.0"
        }

        fn init(&mut self, _ctx: &mut PluginContext<'_>) {
            self.init_called = true;
        }
    }

    #[test]
    fn plugin_registry() {
        let mut registry = PluginRegistry::new();
        registry.register(TestPlugin::new());
        assert_eq!(registry.len(), 1);
        assert_eq!(registry.list(), vec![("test-plugin", "1.0.0")]);

        let mut ontology = OntologyRegistry::new();
        let mut i18n = I18n::new("en");
        let mut theme = Theme::dark();
        let mut ctx = PluginContext {
            ontology: &mut ontology,
            i18n: &mut i18n,
            theme: &mut theme,
        };
        registry.init_all(&mut ctx);
    }
}
