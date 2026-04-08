# agpu Roadmap

Tracking the development of agpu — a standalone agentic-first GPU rendering backend.

## v1.0.0 — Core Foundation (Complete)

### GPU Backend

- [x] `GpuContext` with Vulkan-first, automatic fallback to OpenGL/GLES/platform default
- [x] `BackendPreference` enum (Vulkan, OpenGL, PlatformDefault, Specific)
- [x] Surface management (`GpuSurface`, `GpuSurfaceTexture`)
- [x] Full resource wrappers: buffer, texture, texture view, sampler, shader module
- [x] Pipeline wrappers: render pipeline, compute pipeline, pipeline layout, bind group
- [x] Command encoding: encoder, render pass, compute pass, command buffer
- [x] Custom WGSL triangle-strip batched shape shader

### 2D Rendering

- [x] `ShapeRenderer` — batched 2D geometry in a single draw call
- [x] Fill and stroke: rectangles (sharp and rounded), circles, lines
- [x] Dynamic viewport resize
- [x] Vertex-based rendering with `bytemuck` derive

### Text Rendering

- [x] `TextEngine` powered by glyphon (harfbuzz shaping, subpixel positioning)
- [x] Multi-line text layout with clip-rect support
- [x] Font weight mapping via `FontWeight` enum
- [x] GPU text atlas management with `trim()` for memory cleanup

### Paint Abstraction

- [x] `Painter` trait — backend-agnostic 2D drawing API
- [x] `AgpuPainter` — wgpu-backed implementation with clip stack
- [x] `NullPainter` — headless/test mode (intentionally empty operations)
- [x] Text measurement estimation (`measure_text`)

### Elm Architecture Runtime

- [x] `Model` trait with `update`, `view`, `handle_event`, `init`, `title`
- [x] `Command` enum: None, Quit, Batch, Message, Task, TaskWithTimeout, TaskCancellable
- [x] `Command::SetTickRate` — dynamic tick interval control
- [x] `Command::AgentAction` — validated agent action dispatch via ontology
- [x] `Command::ExportOntology` — re-export agent metadata
- [x] `Frame` abstraction with painter access, widget registration, and hit-map
- [x] `CancellationToken` for cooperative task cancellation
- [x] `ProgramOptions` with window size, vsync, tick rate, backend, fullscreen, transparency

### Application Runner

- [x] `AgpuApp` builder with `with_options()` and `with_backend()`
- [x] winit `ApplicationHandler` state machine (Uninitialised → Running → Exited)
- [x] Tick-rate-aware `about_to_wait` — only redraws at the configured interval
- [x] Tick event dispatch to model at each interval
- [x] Cursor position tracking and mouse event patching
- [x] Surface error recovery (Lost, Outdated, OutOfMemory)

### Event System

- [x] Unified `Event` enum: Key, Mouse, TextInput, Resize, Focus, Tick, FileDrop, DragDrop, AgentAction
- [x] `KeyEvent` with code, modifiers, kind (Press/Release/Repeat)
- [x] `MouseEvent` with position, button, kind (Click/Release/Drag/Move/Scroll)
- [x] `DragDropEvent` with payloads (Text, Index, Path, Json)
- [x] `HitMap` for z-ordered position-to-widget routing
- [x] winit → agpu event conversion for all window event types

### Ontology (Agent Discoverability)

- [x] `Discoverable` trait: schema, capabilities, actions, semantic_role, agent_state, execute_action
- [x] `WidgetSchema` with properties, actions, tags, default role
- [x] `PropertySchema` with type, constraints (min/max/step/pattern/allowed values)
- [x] `AgentAction` with typed parameters and validation (`validate_params`)
- [x] `AgentCapability` enum (Focusable, Scrollable, Editable, Selectable, etc.)
- [x] `SemanticRole` enum (Button, TextInput, Slider, Container, etc.)
- [x] `OntologyRegistry` with type catalog, search, and live UI tree
- [x] `UiTree` / `UiNode` with depth-first search, role filtering, capability filtering
- [x] `Accessibility` struct (ARIA-style attributes for screen readers and agents)
- [x] GPU ontology wrappers: `AgpuAppOntology`, `PipelineOntology`, `SurfaceOntology`, etc.
- [x] Agent action validation through the ontology registry before dispatch

### Core Types

- [x] `Color` (f32 RGBA, named constants, lerp, from_rgb8/rgba8)
- [x] `Position`, `Size`, `Rect` (contains, intersect, union, inflate, translate)
- [x] `Margin` (uniform and per-side)
- [x] `TextStyle` (font size, color, weight, line height)
- [x] `FontWeight` (Thin through Black)
- [x] Serde serialization for all core types

### Testing

- [x] 94 unit tests across 8 modules
- [x] Zero clippy warnings
- [x] Clean debug and release builds

---

## v1.1.0 — Rendering Improvements (Complete)

- [x] **Stroked rounded rectangles**: Proper inner-cutout rendering (currently falls back to filled rounded rect when `corner_radius > 0.5` — requires background color or stencil buffer)
- [x] **Precise text measurement**: `measure_text` currently estimates width as `font_size × 0.6 × char_count`; integrate actual glyph metrics by using interior mutability in `TextEngine`
- [x] **Anti-aliasing**: MSAA or shader-based AA for shape edges
- [x] **Gradient fills**: Linear and radial gradient support in the shape renderer
- [x] **Shadow/glow effects**: Drop shadows and outer glow for shapes
- [x] **Image rendering**: Texture-backed image drawing through the `Painter` trait

## v1.2.0 — Runtime Enhancements (Complete)

- [x] **Async task runtime**: Replace `std::thread::spawn` with a proper async executor (tokio or smol) for `Task`, `TaskWithTimeout`, `TaskCancellable`
- [x] **Subscription system**: Elm-style subscriptions for timers, websockets, file watchers
- [x] **Navigation/routing**: Multi-page application support with URL-style routing
- [x] **Hot reload**: Watch for model changes and reload without full restart
- [x] **Input method editor (IME)**: Proper IME support for CJK text input

## v1.3.0 — Widget Library (Complete)

- [x] **Button**: Click handler, disabled state, icon support
- [x] **TextInput**: Single-line editable text with cursor, selection, clipboard
- [x] **TextArea**: Multi-line editable text with scrolling
- [x] **Checkbox / Radio**: Toggle and exclusive-choice controls
- [x] **Slider**: Continuous and discrete value selection
- [x] **Select / Dropdown**: Single and multi-select with search
- [x] **List / Table**: Virtualized scrolling for large datasets
- [x] **Tabs / Panel**: Tabbed and collapsible container layouts
- [x] **Modal / Tooltip**: Overlay and hover-triggered popups
- [x] **Tree view**: Hierarchical data display with expand/collapse
- [x] **Progress bar**: Determinate and indeterminate progress indicators
- [x] **Menu**: Context menus and menu bars

## v1.4.0 — Layout Engine (Complete)

- [x] **Flexbox layout**: Row/column layout with alignment, wrapping, and spacing
- [x] **Grid layout**: 2D grid with row/column spans
- [x] **Constraint-based layout**: Auto-layout with min/max constraints
- [x] **Scroll containers**: Overflow handling with virtual scroll
- [x] **Responsive breakpoints**: Adapt layout to window size

## v2.0.0 — Advanced Features (Complete)

- [x] **3D rendering**: Basic 3D scene support (camera, meshes, lighting)
- [x] **Compute shaders**: General-purpose GPU compute through the ontology
- [x] **Multi-window**: Multiple windows from a single application
- [x] **Accessibility**: Full platform accessibility integration (Windows UI Automation, macOS Accessibility, AT-SPI)
- [x] **Theming**: Complete theme system with dark/light modes and custom palettes
- [x] **Animation framework**: Declarative animations with easing functions
- [x] **Plugin system**: Loadable widget and renderer plugins
