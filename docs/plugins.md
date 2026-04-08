# Plugin System

Dewey provides a plugin system for extending the framework with custom widgets,
themes, translations, and agent capabilities.

## Implementing a Plugin

```rust
use dewey::plugin::{Plugin, PluginContext};

struct MyPlugin;

impl Plugin for MyPlugin {
    fn name(&self) -> &str {
        "my-plugin"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn init(&mut self, ctx: &mut PluginContext<'_>) {
        // Register ontology schemas, i18n catalogs, or modify the theme
        log::info!("MyPlugin initialized");
    }

    fn on_frame(&mut self) {
        // Called every frame (optional)
    }

    fn on_shutdown(&mut self) {
        // Cleanup (optional)
    }
}
```

## Plugin Context

During `init`, plugins receive a `PluginContext` that provides mutable access to:

| Field      | Type                    | Purpose                               |
| ---------- | ----------------------- | ------------------------------------- |
| `ontology` | `&mut OntologyRegistry` | Register widget schemas               |
| `i18n`     | `&mut I18n`             | Add message catalogs and translations |
| `theme`    | `&mut Theme`            | Extend or override theme tokens       |

## Registering Plugins

```rust
use dewey::plugin::PluginRegistry;

let mut registry = PluginRegistry::new();
registry.register(MyPlugin);
```

The `PluginRegistry` manages the lifecycle:

- `init_all(ctx)` — calls `init` on every registered plugin
- `on_frame()` — calls the per-frame hook on all plugins
- `on_shutdown()` — calls the shutdown hook on all plugins

## Use Cases

- **Widget packs**: register custom widget schemas into the ontology
- **Theme packs**: apply a branded color scheme
- **Localization packs**: load additional message catalogs via the i18n system
- **Telemetry**: collect frame timing data in `on_frame`
