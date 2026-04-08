//! Event system and winit conversion for agpu.
//!
//! Provides a unified event type and translates keyboard, mouse, window,
//! and text input events from winit into agpu's own `Event` enum.

use crate::core::{Position, Rect, Size};
use serde::{Deserialize, Serialize};
use winit::event::{ElementState, MouseButton as WinitButton, WindowEvent};
use winit::keyboard::{Key, NamedKey};

// ── Event Types ─────────────────────────────────────────────────────

/// A GUI event.
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    /// Keyboard event.
    Key(KeyEvent),
    /// Mouse event.
    Mouse(MouseEvent),
    /// Text input (already composed characters).
    TextInput(String),
    /// Window was resized to the given logical size.
    Resize(Size),
    /// Window gained focus.
    FocusGained,
    /// Window lost focus.
    FocusLost,
    /// Window close requested.
    CloseRequested,
    /// A scheduled tick from the runtime (for animations).
    Tick,
    /// File(s) dropped onto the window.
    FileDrop(Vec<String>),
    /// File(s) hovering over the window.
    FileHover(Vec<String>),
    /// File hover cancelled.
    FileHoverCancelled,
    /// Drag-and-drop event.
    DragDrop(DragDropEvent),
    /// An agent action dispatched via `Command::AgentAction`.
    AgentAction {
        agent_id: String,
        action: String,
        params: serde_json::Value,
    },
    /// Input method editor (IME) preedit text (for CJK input).
    ImePreedit {
        /// Current composition text (empty when composition ends).
        text: String,
        /// Cursor position within the preedit text, if known.
        cursor: Option<(usize, usize)>,
    },
    /// IME composition committed — final text to insert.
    ImeCommit(String),
}

/// A keyboard event.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyEvent {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
    pub kind: KeyEventKind,
}

impl KeyEvent {
    pub fn new(code: KeyCode, modifiers: KeyModifiers) -> Self {
        Self {
            code,
            modifiers,
            kind: KeyEventKind::Press,
        }
    }

    pub fn is_ctrl(&self, code: KeyCode) -> bool {
        self.code == code && self.modifiers.contains(KeyModifiers::CONTROL)
    }

    pub fn is_key(&self, code: KeyCode) -> bool {
        self.code == code && self.modifiers.is_empty()
    }
}

/// The kind of key event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyEventKind {
    Press,
    Release,
    Repeat,
}

/// Key code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KeyCode {
    Char(char),
    F(u8),
    Backspace,
    Enter,
    Tab,
    BackTab,
    Esc,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
    Insert,
    Delete,
    Null,
}

/// Key modifier flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct KeyModifiers(u8);

impl KeyModifiers {
    pub const NONE: Self = Self(0);
    pub const SHIFT: Self = Self(1 << 0);
    pub const CONTROL: Self = Self(1 << 1);
    pub const ALT: Self = Self(1 << 2);
    pub const SUPER: Self = Self(1 << 3);

    pub const fn empty() -> Self {
        Self(0)
    }

    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

impl std::ops::BitOr for KeyModifiers {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for KeyModifiers {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

/// A mouse event.
#[derive(Debug, Clone, PartialEq)]
pub struct MouseEvent {
    pub kind: MouseEventKind,
    pub position: Position,
    pub modifiers: KeyModifiers,
}

impl MouseEvent {
    pub fn is_click(&self) -> bool {
        matches!(self.kind, MouseEventKind::Click(_))
    }

    pub fn is_drag(&self) -> bool {
        matches!(self.kind, MouseEventKind::Drag(_))
    }

    pub fn is_scroll(&self) -> bool {
        matches!(self.kind, MouseEventKind::Scroll { .. })
    }

    /// Whether the given rect was clicked.
    pub fn clicked_in(&self, area: Rect) -> bool {
        self.is_click() && area.contains(self.position)
    }
}

/// The kind of mouse event.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MouseEventKind {
    Click(MouseButton),
    Release(MouseButton),
    Drag(MouseButton),
    Move,
    Scroll { delta_x: f32, delta_y: f32 },
}

/// Mouse button.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

/// A drag-and-drop event.
#[derive(Debug, Clone, PartialEq)]
pub struct DragDropEvent {
    /// The kind of drag-drop event.
    pub kind: DragDropKind,
    /// Current pointer position during drag.
    pub position: Position,
}

/// The kind of drag-and-drop event.
#[derive(Debug, Clone, PartialEq)]
pub enum DragDropKind {
    /// A drag operation started from a widget.
    DragStart {
        source_id: String,
        payload: DragPayload,
    },
    /// The dragged item is hovering over a potential drop target.
    DragOver { target_id: String },
    /// The dragged item left a potential drop target.
    DragLeave { target_id: String },
    /// The dragged item was dropped on a target.
    Drop {
        source_id: String,
        target_id: String,
        payload: DragPayload,
    },
    /// The drag operation was cancelled.
    DragCancel,
}

/// Data carried during a drag-and-drop operation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DragPayload {
    /// Plain text content.
    Text(String),
    /// An item index (e.g. from a list or table).
    Index(usize),
    /// A path in a tree widget.
    Path(Vec<String>),
    /// Arbitrary JSON data.
    Json(serde_json::Value),
}

/// A hit-test map that resolves positions to widget IDs.
///
/// Widgets register their bounds during rendering; the runtime uses this
/// to route mouse events to the correct widget.
#[derive(Debug, Default)]
pub struct HitMap {
    entries: Vec<HitEntry>,
}

#[derive(Debug)]
struct HitEntry {
    agent_id: String,
    bounds: Rect,
    z_order: u32,
}

impl HitMap {
    pub fn new() -> Self {
        Self::default()
    }

    /// Clear all entries (called at the start of each frame).
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Register a widget's clickable bounds.
    pub fn register(&mut self, agent_id: impl Into<String>, bounds: Rect, z_order: u32) {
        self.entries.push(HitEntry {
            agent_id: agent_id.into(),
            bounds,
            z_order,
        });
    }

    /// Find the widget at the given position (highest z-order wins).
    pub fn hit_test(&self, pos: Position) -> Option<&str> {
        self.entries
            .iter()
            .filter(|e| e.bounds.contains(pos))
            .max_by_key(|e| e.z_order)
            .map(|e| e.agent_id.as_str())
    }
}

// ── Winit Conversion ────────────────────────────────────────────────

/// Convert a single winit `WindowEvent` into zero or more agpu events.
pub fn convert_window_event(event: &WindowEvent) -> Vec<Event> {
    let mut out = Vec::new();

    match event {
        WindowEvent::KeyboardInput { event: key_ev, .. } => {
            if let Some(code) = convert_key(&key_ev.logical_key) {
                let kind = match key_ev.state {
                    ElementState::Pressed => KeyEventKind::Press,
                    ElementState::Released => KeyEventKind::Release,
                };
                let mods = KeyModifiers::empty(); // winit modifiers handled below
                out.push(Event::Key(KeyEvent {
                    code,
                    modifiers: mods,
                    kind,
                }));
            }
            // Generate TextInput for printable characters on press
            if key_ev.state == ElementState::Pressed {
                if let Key::Character(ch) = &key_ev.logical_key {
                    out.push(Event::TextInput(ch.to_string()));
                }
            }
        }

        WindowEvent::CursorMoved { position, .. } => {
            out.push(Event::Mouse(MouseEvent {
                kind: MouseEventKind::Move,
                position: Position::new(position.x as f32, position.y as f32),
                modifiers: KeyModifiers::empty(),
            }));
        }

        WindowEvent::MouseInput { state, button, .. } => {
            let btn = match button {
                WinitButton::Left => MouseButton::Left,
                WinitButton::Right => MouseButton::Right,
                WinitButton::Middle => MouseButton::Middle,
                _ => MouseButton::Left,
            };
            let kind = match state {
                ElementState::Pressed => MouseEventKind::Click(btn),
                ElementState::Released => MouseEventKind::Release(btn),
            };
            out.push(Event::Mouse(MouseEvent {
                kind,
                position: Position::ZERO, // updated by cursor position tracking
                modifiers: KeyModifiers::empty(),
            }));
        }

        WindowEvent::MouseWheel { delta, .. } => {
            let (dx, dy) = match delta {
                winit::event::MouseScrollDelta::LineDelta(x, y) => (*x, *y),
                winit::event::MouseScrollDelta::PixelDelta(p) => (p.x as f32, p.y as f32),
            };
            out.push(Event::Mouse(MouseEvent {
                kind: MouseEventKind::Scroll {
                    delta_x: dx,
                    delta_y: dy,
                },
                position: Position::ZERO,
                modifiers: KeyModifiers::empty(),
            }));
        }

        WindowEvent::Resized(size) => {
            out.push(Event::Resize(Size::new(
                size.width as f32,
                size.height as f32,
            )));
        }

        WindowEvent::Focused(focused) => {
            out.push(if *focused {
                Event::FocusGained
            } else {
                Event::FocusLost
            });
        }

        WindowEvent::CloseRequested => {
            out.push(Event::CloseRequested);
        }

        WindowEvent::DroppedFile(path) => {
            out.push(Event::FileDrop(vec![path.display().to_string()]));
        }

        WindowEvent::HoveredFile(path) => {
            out.push(Event::FileHover(vec![path.display().to_string()]));
        }

        WindowEvent::HoveredFileCancelled => {
            out.push(Event::FileHoverCancelled);
        }

        WindowEvent::Ime(ime) => match ime {
            winit::event::Ime::Preedit(text, cursor) => {
                out.push(Event::ImePreedit {
                    text: text.clone(),
                    cursor: *cursor,
                });
            }
            winit::event::Ime::Commit(text) => {
                out.push(Event::ImeCommit(text.clone()));
            }
            _ => {}
        },

        _ => {}
    }

    out
}

/// Convert winit logical key to agpu `KeyCode`.
fn convert_key(key: &Key) -> Option<KeyCode> {
    match key {
        Key::Named(named) => match named {
            NamedKey::Enter => Some(KeyCode::Enter),
            NamedKey::Tab => Some(KeyCode::Tab),
            NamedKey::Backspace => Some(KeyCode::Backspace),
            NamedKey::Escape => Some(KeyCode::Esc),
            NamedKey::ArrowLeft => Some(KeyCode::Left),
            NamedKey::ArrowRight => Some(KeyCode::Right),
            NamedKey::ArrowUp => Some(KeyCode::Up),
            NamedKey::ArrowDown => Some(KeyCode::Down),
            NamedKey::Home => Some(KeyCode::Home),
            NamedKey::End => Some(KeyCode::End),
            NamedKey::PageUp => Some(KeyCode::PageUp),
            NamedKey::PageDown => Some(KeyCode::PageDown),
            NamedKey::Insert => Some(KeyCode::Insert),
            NamedKey::Delete => Some(KeyCode::Delete),
            NamedKey::F1 => Some(KeyCode::F(1)),
            NamedKey::F2 => Some(KeyCode::F(2)),
            NamedKey::F3 => Some(KeyCode::F(3)),
            NamedKey::F4 => Some(KeyCode::F(4)),
            NamedKey::F5 => Some(KeyCode::F(5)),
            NamedKey::F6 => Some(KeyCode::F(6)),
            NamedKey::F7 => Some(KeyCode::F(7)),
            NamedKey::F8 => Some(KeyCode::F(8)),
            NamedKey::F9 => Some(KeyCode::F(9)),
            NamedKey::F10 => Some(KeyCode::F(10)),
            NamedKey::F11 => Some(KeyCode::F(11)),
            NamedKey::F12 => Some(KeyCode::F(12)),
            _ => None,
        },
        Key::Character(ch) => {
            let c = ch.chars().next()?;
            Some(KeyCode::Char(c))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── KeyModifiers ─────────────────────────────────────────────

    #[test]
    fn key_modifiers_empty() {
        let m = KeyModifiers::empty();
        assert!(m.is_empty());
        assert!(!m.contains(KeyModifiers::SHIFT));
    }

    #[test]
    fn key_modifiers_contains() {
        let m = KeyModifiers::SHIFT | KeyModifiers::CONTROL;
        assert!(m.contains(KeyModifiers::SHIFT));
        assert!(m.contains(KeyModifiers::CONTROL));
        assert!(!m.contains(KeyModifiers::ALT));
    }

    #[test]
    fn key_modifiers_union() {
        let a = KeyModifiers::SHIFT;
        let b = KeyModifiers::ALT;
        let u = a.union(b);
        assert!(u.contains(KeyModifiers::SHIFT));
        assert!(u.contains(KeyModifiers::ALT));
    }

    #[test]
    fn key_modifiers_bitor_assign() {
        let mut m = KeyModifiers::NONE;
        m |= KeyModifiers::SUPER;
        assert!(m.contains(KeyModifiers::SUPER));
    }

    // ── KeyEvent ─────────────────────────────────────────────────

    #[test]
    fn key_event_new() {
        let ke = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        assert_eq!(ke.code, KeyCode::Enter);
        assert!(ke.modifiers.is_empty());
        assert_eq!(ke.kind, KeyEventKind::Press);
    }

    #[test]
    fn key_event_is_ctrl() {
        let ke = KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL);
        assert!(ke.is_ctrl(KeyCode::Char('s')));
        assert!(!ke.is_ctrl(KeyCode::Char('c')));
    }

    #[test]
    fn key_event_is_key() {
        let ke = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        assert!(ke.is_key(KeyCode::Esc));

        let with_mod = KeyEvent::new(KeyCode::Esc, KeyModifiers::SHIFT);
        assert!(!with_mod.is_key(KeyCode::Esc));
    }

    // ── MouseEvent ───────────────────────────────────────────────

    #[test]
    fn mouse_event_is_click() {
        let me = MouseEvent {
            kind: MouseEventKind::Click(MouseButton::Left),
            position: Position::ZERO,
            modifiers: KeyModifiers::NONE,
        };
        assert!(me.is_click());
        assert!(!me.is_drag());
        assert!(!me.is_scroll());
    }

    #[test]
    fn mouse_event_clicked_in() {
        let area = Rect::new(10.0, 10.0, 100.0, 100.0);
        let inside = MouseEvent {
            kind: MouseEventKind::Click(MouseButton::Left),
            position: Position::new(50.0, 50.0),
            modifiers: KeyModifiers::NONE,
        };
        let outside = MouseEvent {
            kind: MouseEventKind::Click(MouseButton::Left),
            position: Position::new(5.0, 5.0),
            modifiers: KeyModifiers::NONE,
        };
        let not_click = MouseEvent {
            kind: MouseEventKind::Move,
            position: Position::new(50.0, 50.0),
            modifiers: KeyModifiers::NONE,
        };
        assert!(inside.clicked_in(area));
        assert!(!outside.clicked_in(area));
        assert!(!not_click.clicked_in(area));
    }

    // ── HitMap ───────────────────────────────────────────────────

    #[test]
    fn hit_map_empty() {
        let hm = HitMap::new();
        assert!(hm.hit_test(Position::ZERO).is_none());
    }

    #[test]
    fn hit_map_register_and_hit() {
        let mut hm = HitMap::new();
        hm.register("btn-1", Rect::new(0.0, 0.0, 50.0, 50.0), 0);
        assert_eq!(hm.hit_test(Position::new(25.0, 25.0)), Some("btn-1"));
        assert!(hm.hit_test(Position::new(60.0, 60.0)).is_none());
    }

    #[test]
    fn hit_map_z_order() {
        let mut hm = HitMap::new();
        hm.register("back", Rect::new(0.0, 0.0, 100.0, 100.0), 0);
        hm.register("front", Rect::new(0.0, 0.0, 100.0, 100.0), 10);
        assert_eq!(hm.hit_test(Position::new(50.0, 50.0)), Some("front"));
    }

    #[test]
    fn hit_map_clear() {
        let mut hm = HitMap::new();
        hm.register("widget", Rect::new(0.0, 0.0, 50.0, 50.0), 0);
        assert!(hm.hit_test(Position::new(25.0, 25.0)).is_some());
        hm.clear();
        assert!(hm.hit_test(Position::new(25.0, 25.0)).is_none());
    }

    // ── DragPayload ──────────────────────────────────────────────

    #[test]
    fn drag_payload_serialize_roundtrip() {
        let payloads = vec![
            DragPayload::Text("hello".into()),
            DragPayload::Index(42),
            DragPayload::Path(vec!["a".into(), "b".into()]),
            DragPayload::Json(serde_json::json!({"key": "value"})),
        ];
        for p in payloads {
            let json = serde_json::to_string(&p).unwrap();
            let p2: DragPayload = serde_json::from_str(&json).unwrap();
            assert_eq!(p, p2);
        }
    }

    // ── KeyCode serialize ────────────────────────────────────────

    #[test]
    fn keycode_serialize_roundtrip() {
        let codes = vec![
            KeyCode::Char('a'),
            KeyCode::F(5),
            KeyCode::Enter,
            KeyCode::Esc,
            KeyCode::Null,
        ];
        for code in codes {
            let json = serde_json::to_string(&code).unwrap();
            let code2: KeyCode = serde_json::from_str(&json).unwrap();
            assert_eq!(code, code2);
        }
    }
}
