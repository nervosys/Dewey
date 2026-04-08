# Architecture Guide

This document describes Dewey's internal architecture and module organization.

## Overview

Dewey is an **Elm-architecture** GUI framework with a complete **ontology** layer
that enables AI agents to discover, inspect, and control GUI applications.

```
┌──────────┐   Event    ┌──────────┐  Command   ┌──────────┐
│  Backend │ ────────→ │  Model   │ ────────→ │ Runtime  │
│ (Painter)│           │ (update) │           │ (Program)│
│          │ ←──────── │  (view)  │ ←──────── │          │
└──────────┘   Frame   └──────────┘           └──────────┘
      ↕                      ↕
┌──────────┐         ┌──────────────┐
│  Widgets │         │  Ontology    │
│  (30)    │         │  (Registry)  │
└──────────┘         └──────────────┘
                           ↕
                     ┌──────────┐
                     │  Agent   │
                     │ Protocol │
                     └──────────┘
```

## Data Flow

1. **Events** arrive from the backend (keyboard, mouse, touch, file drop).
2. The `Model::handle_event` method converts events into typed `Msg` values.
3. `Model::update` processes each `Msg`, mutating state and optionally returning a `Command`.
4. `Model::view` renders the current state into a `Frame` using `Widget::render` calls.
5. The `Painter` trait translates abstract render ops into backend-specific drawing.
6. `Command::Task` spawns asynchronous work that can feed results back as messages.

## Module Map

| Module      | Path               | Purpose                                                                                     |
| ----------- | ------------------ | ------------------------------------------------------------------------------------------- |
| `core`      | `src/core/`        | Rect, Position, Size, Color, Style primitives                                               |
| `widget`    | `src/widget/`      | 30 widgets with Discoverable trait impls                                                    |
| `ontology`  | `src/ontology/`    | Schema, capabilities, actions, registry, UiTree                                             |
| `agent`     | `src/agent/`       | JSON Lines protocol, RPC, HeadlessDriver, session                                           |
| `runtime`   | `src/runtime/`     | Model trait, Command enum, Program runner                                                   |
| `event`     | `src/event/`       | KeyEvent, MouseEvent, Event enum                                                            |
| `layout`    | `src/layout/`      | Direction, Constraint, Layout engine                                                        |
| `animation` | `src/animation/`   | 34 easing functions, Tween, Spring, Timeline, Keyframes                                     |
| `backend`   | `src/backend/`     | EguiPainter (wgpu), TestBackend (record ops)                                                |
| `agpu`      | `agpu/src/`        | Vulkan-first GPU backend — complete wgpu replacement with resource abstraction and ontology |
| `paint`     | `src/paint.rs`     | Painter trait — 9 abstract drawing primitives                                               |
| `focus`     | `src/focus.rs`     | Tab-navigation focus ring                                                                   |
| `overlay`   | `src/overlay.rs`   | Layered modal/dialog rendering                                                              |
| `theme`     | `src/theme.rs`     | Token-based theming, ThemeWatcher, JSON I/O                                                 |
| `i18n`      | `src/i18n.rs`      | Internationalization, MessageCatalog, locale fallback                                       |
| `plugin`    | `src/plugin.rs`    | Plugin trait, PluginRegistry, PluginContext                                                 |
| `dialog`    | `src/dialog.rs`    | Native file/message dialogs (DialogBackend trait)                                           |
| `gpu`       | `src/gpu.rs`       | GPU-accelerated canvas, RenderBatch, quad merging                                           |
| `memory`    | `src/memory.rs`    | Arena allocator, VecPool, InlineString                                                      |
| `profiling` | `src/profiling.rs` | Profiler, FrameProfile, FPS tracking                                                        |
| `tray`      | `src/tray.rs`      | System tray integration (TrayBackend trait)                                                 |
| `window`    | `src/window.rs`    | Multi-window support (WindowManager, WindowConfig)                                          |
| `util`      | `src/util/`        | Fuzzy matching, undo/redo, state persistence                                                |
| `error`     | `src/error.rs`     | DeweyError, DeweyResult                                                                     |

## The Painter Trait

All widget rendering goes through `Painter`, making backends fully pluggable:

```rust
pub trait Painter {
    fn fill_rect(&mut self, rect: Rect, color: Color);
    fn stroke_rect(&mut self, rect: Rect, color: Color, width: f32);
    fn fill_circle(&mut self, cx: f32, cy: f32, radius: f32, color: Color);
    fn stroke_circle(&mut self, cx: f32, cy: f32, radius: f32, color: Color, width: f32);
    fn line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, color: Color, width: f32);
    fn text(&mut self, x: f32, y: f32, text: &str, size: f32, color: Color);
    fn measure_text(&self, text: &str, size: f32) -> (f32, f32);
    fn push_clip(&mut self, rect: Rect);
    fn pop_clip(&mut self);
}
```

Built-in implementations:
- **`EguiPainter`** — GPU-accelerated via egui/wgpu (default backend)
- **`AgpuPainter`** — Vulkan-first GPU rendering with complete resource abstraction (`agpu` crate)
- **`NullPainter`** — No-op, for headless agent driving
- **`TestPainter`** — Records render ops for assertion-based testing
- **`WebPainter`** — Canvas 2D for wasm32 targets
- **`ImagePainter`** — Software rasterizer producing pixel buffers

## The Ontology Layer

Every widget implements `Discoverable`:

```rust
pub trait Discoverable {
    fn agent_id(&self) -> Option<&str>;
    fn schema(&self) -> WidgetSchema;
    fn semantic_role(&self) -> SemanticRole;
    fn capabilities(&self) -> Vec<AgentCapability>;
    fn actions(&self) -> Vec<AgentAction>;
    fn agent_state(&self) -> serde_json::Value;
    fn execute_action(&mut self, name: &str, params: &serde_json::Value) -> Result<...>;
    fn accessibility(&self) -> Accessibility;
}
```

The `OntologyRegistry` aggregates all widget schemas and supports querying by
name, role, and keyword. The `UiTree` provides a hierarchical snapshot of the
full widget tree for agent inspection.

## Agent Protocol

Communication uses JSON Lines on stdin/stdout (or WebSocket with the
`ws-transport` feature). See [agent-protocol.md](agent-protocol.md) for the
full specification.

## Feature Flags

| Feature        | Default | Description                                 |
| -------------- | ------- | ------------------------------------------- |
| `egui-backend` | Yes     | GPU-accelerated rendering via egui/wgpu     |
| `ws-transport` | No      | WebSocket transport for agent communication |

## Companion Crates

| Crate  | Path    | Description                                                                                 |
| ------ | ------- | ------------------------------------------------------------------------------------------- |
| `agpu` | `agpu/` | Vulkan-first GPU backend — complete wgpu replacement with resource abstraction and ontology |
