# agpu

**Standalone agentic-first GPU rendering backend — a complete wgpu replacement with built-in ontology for AI agent discoverability.**

## Features

- **Vulkan-first** with automatic fallback to OpenGL/GLES and platform defaults
- **Complete ontology** — every GPU resource, pipeline, and UI element is discoverable by AI agents
- **Elm architecture** runtime (`Model` / `update` / `view`) with async commands and cancellation
- **Painter trait** — backend-agnostic 2D drawing (shapes, text, clipping)
- **GPU text** via glyphon (harfbuzz shaping, subpixel positioning)
- **Batched shape renderer** — circles, rounded rects, lines in a single draw call
- **Zero `unsafe`** in application code
- **No external GUI framework dependency** — fully self-contained

## Architecture

```text
┌─────────────────────────────────────────────────┐
│  AgpuApp<M: Model>                              │
│  ├── GpuContext (instance/adapter/device/queue) │
│  ├── ShapeRenderer (batched 2D geometry)        │
│  ├── TextEngine (glyphon GPU text)              │
│  ├── AgpuPainter (Painter trait impl)           │
│  └── OntologyRegistry (agent discoverability)   │
└─────────────────────────────────────────────────┘
```

## Quick Start

```rust
use agpu::prelude::*;

struct App { count: i32 }

#[derive(Debug)]
enum Msg { Increment }

impl Model for App {
    type Msg = Msg;

    fn update(&mut self, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::Increment => self.count += 1,
        }
        Command::None
    }

    fn view(&self, frame: &mut Frame<'_>) {
        let p = frame.painter();
        p.fill_rect(frame.area, Color::BLACK, 0.0);
        p.text(
            Position::new(10.0, 10.0),
            &format!("Count: {}", self.count),
            &TextStyle { font_size: 24.0, color: Color::WHITE, ..Default::default() },
        );
    }

    fn handle_event(&self, _event: Event) -> Option<Msg> { None }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    AgpuApp::new(App { count: 0 }).run()
}
```

## Backend Selection

```rust
use agpu::{AgpuApp, BackendPreference};

// Vulkan (default)
AgpuApp::new(app).run();

// Prefer OpenGL / GLES
AgpuApp::new(app)
    .with_backend(BackendPreference::OpenGLPreferred)
    .run();

// Platform default (D3D12 on Windows, Metal on macOS, Vulkan on Linux)
AgpuApp::new(app)
    .with_backend(BackendPreference::PlatformDefault)
    .run();
```

## Ontology

Every GPU resource implements the `Discoverable` trait, exposing:

| Method             | Description                                            |
| ------------------ | ------------------------------------------------------ |
| `schema()`         | JSON-serializable widget/resource schema               |
| `capabilities()`   | Advertised capabilities (`Focusable`, `Scrollable`, …) |
| `actions()`        | Available agent actions with typed parameters          |
| `semantic_role()`  | ARIA-like semantic role (`Button`, `TextInput`, …)     |
| `agent_state()`    | Live key-value state snapshot                          |
| `execute_action()` | Programmatic action execution                          |

The `OntologyRegistry` aggregates all schemas and maintains a live `UiTree` for frame-by-frame agent inspection.

## License

AGPL-3.0-or-later
