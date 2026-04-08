//! Elm-architecture runtime for agpu applications.
//!
//! - **Model**: Application state
//! - **Message**: Events that update state
//! - **Update**: Pure function `(Model, Msg) -> (Model, Command)`
//! - **View**: Pure function `Model -> UI description`

use std::time::Duration;

use crate::core::Rect;
use crate::ontology::OntologyRegistry;

/// A token that can be checked to determine if a task should be cancelled.
#[derive(Clone)]
pub struct CancellationToken {
    cancelled: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl CancellationToken {
    /// Create a new cancellation token.
    pub fn new() -> Self {
        Self {
            cancelled: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    /// Check if cancellation has been requested.
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Request cancellation.
    pub fn cancel(&self) {
        self.cancelled
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }
}

impl Default for CancellationToken {
    fn default() -> Self {
        Self::new()
    }
}

/// A command returned from [`Model::update`] to request side effects.
pub enum Command<Msg> {
    /// No operation.
    None,
    /// Quit the application.
    Quit,
    /// Execute multiple commands.
    Batch(Vec<Command<Msg>>),
    /// Produce a message asynchronously after the current update.
    Message(Msg),
    /// Set the tick interval for animation / periodic updates.
    SetTickRate(Duration),
    /// Request that the agent ontology registry be exported to JSON.
    ExportOntology,
    /// Execute an agent action on a widget identified by agent_id.
    AgentAction {
        agent_id: String,
        action: String,
        params: serde_json::Value,
    },
    /// Spawn an asynchronous task that eventually produces a message.
    Task(Box<dyn FnOnce() -> Msg + Send>),
    /// Spawn an async task with a timeout. If the task doesn't complete
    /// within the given duration, the timeout message is delivered instead.
    TaskWithTimeout {
        task: Box<dyn FnOnce() -> Msg + Send>,
        timeout: Duration,
        on_timeout: Msg,
    },
    /// Spawn a cancellable async task. The closure receives a [`CancellationToken`]
    /// that it can poll to exit early.
    TaskCancellable {
        task: Box<dyn FnOnce(CancellationToken) -> Msg + Send>,
        token: CancellationToken,
    },
}

impl<Msg: std::fmt::Debug> std::fmt::Debug for Command<Msg> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "None"),
            Self::Quit => write!(f, "Quit"),
            Self::Batch(cmds) => f.debug_tuple("Batch").field(cmds).finish(),
            Self::Message(msg) => f.debug_tuple("Message").field(msg).finish(),
            Self::SetTickRate(d) => f.debug_tuple("SetTickRate").field(d).finish(),
            Self::ExportOntology => write!(f, "ExportOntology"),
            Self::AgentAction {
                agent_id,
                action,
                params,
            } => f
                .debug_struct("AgentAction")
                .field("agent_id", agent_id)
                .field("action", action)
                .field("params", params)
                .finish(),
            Self::Task(_) => write!(f, "Task(<fn>)"),
            Self::TaskWithTimeout { timeout, .. } => {
                write!(f, "TaskWithTimeout({}ms)", timeout.as_millis())
            }
            Self::TaskCancellable { .. } => write!(f, "TaskCancellable(<fn>)"),
        }
    }
}

/// The core trait for application models (Elm Architecture).
pub trait Model: Sized {
    /// The message type for this application.
    type Msg: Send + 'static;

    /// Handle a message and return an updated model plus optional command.
    fn update(&mut self, msg: Self::Msg) -> Command<Self::Msg>;

    /// Render the model into the GUI frame.
    fn view(&self, frame: &mut Frame<'_>);

    /// Convert a raw event into an application message.
    /// Return `None` to ignore the event.
    fn handle_event(&self, event: crate::event::Event) -> Option<Self::Msg>;

    /// Called once at startup. Return an initial command.
    fn init(&self) -> Command<Self::Msg> {
        Command::None
    }

    /// Called when the agent ontology is exported. Override to customize.
    fn register_ontology(&self, _registry: &mut OntologyRegistry) {}

    /// Application title (used as window title).
    fn title(&self) -> &str {
        "agpu App"
    }

    /// Return subscriptions for this model. Called after each update.
    fn subscriptions(&self) -> Vec<Subscription<Self::Msg>> {
        Vec::new()
    }

    /// Return the current route. Override for multi-page apps.
    fn current_route(&self) -> &str {
        "/"
    }
}

/// A rendering frame — abstraction over the GUI backend.
///
/// During `Model::view`, the frame provides methods to draw widgets
/// and manage the UI tree for agent discoverability.
pub struct Frame<'a> {
    /// The available drawing area.
    pub area: Rect,
    /// The hit map for mouse routing.
    pub hit_map: &'a mut crate::event::HitMap,
    /// The ontology tree being built for this frame.
    ui_nodes: Vec<crate::ontology::UiNode>,
    /// The painter for this frame.
    painter: &'a mut dyn crate::paint::Painter,
}

impl<'a> Frame<'a> {
    /// Create a new frame with the given area, hit map, and painter.
    pub fn new(
        area: Rect,
        hit_map: &'a mut crate::event::HitMap,
        painter: &'a mut dyn crate::paint::Painter,
    ) -> Self {
        Self {
            area,
            hit_map,
            ui_nodes: Vec::new(),
            painter,
        }
    }

    /// Get a mutable reference to the painter for this frame.
    pub fn painter(&mut self) -> &mut dyn crate::paint::Painter {
        self.painter
    }

    /// Register a widget in the UI tree for agent discoverability.
    pub fn register_widget(&mut self, node: crate::ontology::UiNode) {
        self.ui_nodes.push(node);
    }

    /// Register a hitbox for mouse event routing.
    pub fn register_hitbox(&mut self, agent_id: impl Into<String>, bounds: Rect, z_order: u32) {
        self.hit_map.register(agent_id, bounds, z_order);
    }

    /// Take the collected UI nodes (consumed by the runtime after rendering).
    pub fn take_nodes(&mut self) -> Vec<crate::ontology::UiNode> {
        std::mem::take(&mut self.ui_nodes)
    }
}

/// Configuration for the application runner.
pub struct ProgramOptions {
    /// Tick interval for animation. `None` disables ticking.
    pub tick_rate: Option<Duration>,
    /// Initial window width in logical pixels.
    pub width: f32,
    /// Initial window height in logical pixels.
    pub height: f32,
    /// Whether to start in fullscreen.
    pub fullscreen: bool,
    /// Whether the window is resizable.
    pub resizable: bool,
    /// Whether to enable vsync.
    pub vsync: bool,
    /// Whether to use a transparent window.
    pub transparent: bool,
    /// GPU backend preference (Vulkan-first by default).
    pub backend: crate::types::BackendPreference,
    /// MSAA sample count (1 = disabled, 4 = recommended).
    pub msaa_samples: u32,
}

impl Default for ProgramOptions {
    fn default() -> Self {
        Self {
            tick_rate: Some(Duration::from_millis(16)), // ~60fps
            width: 800.0,
            height: 600.0,
            fullscreen: false,
            resizable: true,
            vsync: true,
            transparent: false,
            backend: crate::types::BackendPreference::default(),
            msaa_samples: 4,
        }
    }
}

// ── Subscription system ─────────────────────────────────────────────

/// An Elm-style subscription — a source of events managed by the runtime.
pub enum Subscription<Msg> {
    /// Timer subscription: produces a message at the given interval.
    Timer {
        /// Unique identifier for this subscription.
        id: String,
        /// The interval between messages.
        interval: Duration,
        /// Factory function producing the message each tick.
        msg: Box<dyn Fn() -> Msg + Send>,
    },
    /// One-shot delay: produces a single message after the given duration.
    Delay {
        id: String,
        duration: Duration,
        msg: Msg,
    },
}

impl<Msg: std::fmt::Debug> std::fmt::Debug for Subscription<Msg> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Timer { id, interval, .. } => f
                .debug_struct("Timer")
                .field("id", id)
                .field("interval", interval)
                .finish(),
            Self::Delay { id, duration, msg } => f
                .debug_struct("Delay")
                .field("id", id)
                .field("duration", duration)
                .field("msg", msg)
                .finish(),
        }
    }
}

// ── Router ──────────────────────────────────────────────────────────

/// Simple URL-style router for multi-page applications.
#[derive(Debug, Clone)]
pub struct Router {
    current: String,
    history: Vec<String>,
}

impl Router {
    /// Create a new router starting at the given route.
    pub fn new(initial: impl Into<String>) -> Self {
        let initial = initial.into();
        Self {
            current: initial.clone(),
            history: vec![initial],
        }
    }

    /// Navigate to a new route, pushing the current one onto the history stack.
    pub fn navigate(&mut self, route: impl Into<String>) {
        let route = route.into();
        self.history.push(self.current.clone());
        self.current = route;
    }

    /// Go back to the previous route. Returns `true` if successful.
    pub fn back(&mut self) -> bool {
        if let Some(prev) = self.history.pop() {
            self.current = prev;
            true
        } else {
            false
        }
    }

    /// Get the current route.
    pub fn current(&self) -> &str {
        &self.current
    }

    /// Get the depth of the history stack.
    pub fn history_len(&self) -> usize {
        self.history.len()
    }

    /// Match a route pattern against the current route.
    /// Supports simple patterns like "/users/:id" where `:id` matches any segment.
    pub fn matches(&self, pattern: &str) -> Option<Vec<(String, String)>> {
        let route_parts: Vec<&str> = self.current.split('/').collect();
        let pattern_parts: Vec<&str> = pattern.split('/').collect();

        if route_parts.len() != pattern_parts.len() {
            return None;
        }

        let mut params = Vec::new();
        for (r, p) in route_parts.iter().zip(pattern_parts.iter()) {
            if let Some(name) = p.strip_prefix(':') {
                params.push((name.to_string(), r.to_string()));
            } else if r != p {
                return None;
            }
        }
        Some(params)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cancellation_token_not_cancelled_initially() {
        let token = CancellationToken::new();
        assert!(!token.is_cancelled());
    }

    #[test]
    fn cancellation_token_cancel() {
        let token = CancellationToken::new();
        token.cancel();
        assert!(token.is_cancelled());
    }

    #[test]
    fn cancellation_token_clone_shares_state() {
        let token = CancellationToken::new();
        let clone = token.clone();
        token.cancel();
        assert!(clone.is_cancelled());
    }

    #[test]
    fn cancellation_token_default() {
        let token = CancellationToken::default();
        assert!(!token.is_cancelled());
    }

    #[test]
    fn program_options_defaults() {
        let opts = ProgramOptions::default();
        assert_eq!(opts.width, 800.0);
        assert_eq!(opts.height, 600.0);
        assert!(!opts.fullscreen);
        assert!(opts.resizable);
        assert!(opts.vsync);
        assert!(!opts.transparent);
        assert!(opts.tick_rate.is_some());
    }

    #[test]
    fn command_debug_variants() {
        let none: Command<String> = Command::None;
        assert_eq!(format!("{:?}", none), "None");

        let quit: Command<String> = Command::Quit;
        assert_eq!(format!("{:?}", quit), "Quit");

        let msg: Command<String> = Command::Message("hello".into());
        assert!(format!("{:?}", msg).contains("hello"));

        let export: Command<String> = Command::ExportOntology;
        assert_eq!(format!("{:?}", export), "ExportOntology");
    }

    #[test]
    fn frame_take_nodes() {
        let mut hit_map = crate::event::HitMap::new();
        let mut painter = crate::paint::NullPainter;
        let mut frame = Frame::new(
            Rect::new(0.0, 0.0, 800.0, 600.0),
            &mut hit_map,
            &mut painter,
        );

        assert!(frame.take_nodes().is_empty());

        frame.register_widget(crate::ontology::UiNode::new(
            "Button",
            crate::ontology::SemanticRole::Action,
        ));
        let nodes = frame.take_nodes();
        assert_eq!(nodes.len(), 1);
        assert!(frame.take_nodes().is_empty()); // consumed
    }

    #[test]
    fn frame_register_hitbox() {
        let mut hit_map = crate::event::HitMap::new();
        let mut painter = crate::paint::NullPainter;
        let bounds = Rect::new(10.0, 10.0, 50.0, 50.0);
        {
            let mut frame = Frame::new(
                Rect::new(0.0, 0.0, 800.0, 600.0),
                &mut hit_map,
                &mut painter,
            );
            frame.register_hitbox("btn-1", bounds, 0);
        }
        assert_eq!(
            hit_map.hit_test(crate::core::Position::new(30.0, 30.0)),
            Some("btn-1")
        );
    }

    #[test]
    fn router_basic_navigation() {
        let mut router = Router::new("/");
        assert_eq!(router.current(), "/");
        router.navigate("/about");
        assert_eq!(router.current(), "/about");
        assert!(router.back());
        assert_eq!(router.current(), "/");
    }

    #[test]
    fn router_pattern_matching() {
        let router = Router::new("/users/42");
        let params = router.matches("/users/:id").unwrap();
        assert_eq!(params.len(), 1);
        assert_eq!(params[0].0, "id");
        assert_eq!(params[0].1, "42");

        assert!(router.matches("/posts/:id").is_none());
    }

    #[test]
    fn router_history_depth() {
        let mut router = Router::new("/");
        assert_eq!(router.history_len(), 1);
        router.navigate("/a");
        assert_eq!(router.history_len(), 2);
        router.navigate("/b");
        assert_eq!(router.history_len(), 3);
        router.back();
        assert_eq!(router.history_len(), 2);
    }

    #[test]
    fn program_options_msaa_default() {
        let opts = ProgramOptions::default();
        assert_eq!(opts.msaa_samples, 4);
    }

    #[test]
    fn subscription_timer_debug() {
        let sub: Subscription<String> = Subscription::Timer {
            id: "test".into(),
            interval: Duration::from_secs(1),
            msg: Box::new(|| "tick".into()),
        };
        let dbg = format!("{:?}", sub);
        assert!(dbg.contains("Timer"));
        assert!(dbg.contains("test"));
    }
}
