//! LLM Chat — a ChatGPT/Claude-style conversational interface using the agpu GPU backend.
//!
//! Demonstrates:
//! - Dewey's Elm architecture driving a full chat UI
//! - agpu Vulkan-first GPU-accelerated rendering
//! - Multi-message conversation with role-based styling
//! - Simulated LLM streaming responses via async Command::Task
//! - Model selector sidebar (GPT-4o, Claude, Llama, etc.)
//! - System prompt configuration
//! - Scroll area, text input, keyboard shortcuts
//! - Rich layout with sidebar + main content
//! - Ontology registration for agent discoverability
//!
//! Run with:
//!   cargo run --example chat_agpu --features agpu-backend --no-default-features

use dewey::backend::agpu_backend::AgpuProgram;
use dewey::event::KeyEventKind;
use dewey::prelude::*;
use dewey::widget::input::TextInputState;
use dewey::widget::scroll::ScrollState;
use std::cell::RefCell;
use std::time::Duration;

// ── LLM Models ──────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq)]
struct LlmModel {
    id: &'static str,
    name: &'static str,
    provider: &'static str,
    context_window: &'static str,
}

const MODELS: &[LlmModel] = &[
    LlmModel {
        id: "gpt-4o",
        name: "GPT-4o",
        provider: "OpenAI",
        context_window: "128K",
    },
    LlmModel {
        id: "claude-opus-4",
        name: "Claude Opus 4",
        provider: "Anthropic",
        context_window: "200K",
    },
    LlmModel {
        id: "claude-sonnet-4",
        name: "Claude Sonnet 4",
        provider: "Anthropic",
        context_window: "200K",
    },
    LlmModel {
        id: "llama-4-scout",
        name: "Llama 4 Scout",
        provider: "Meta",
        context_window: "10M",
    },
    LlmModel {
        id: "gemini-2.5-pro",
        name: "Gemini 2.5 Pro",
        provider: "Google",
        context_window: "1M",
    },
];

// ── Data model ──────────────────────────────────────────────────────

#[derive(Clone, Debug)]
enum Role {
    User,
    Assistant,
    System,
}

#[derive(Clone, Debug)]
struct ChatMessage {
    role: Role,
    content: String,
    model_name: Option<String>,
    token_count: usize,
}

struct App {
    messages: Vec<ChatMessage>,
    input_state: RefCell<TextInputState>,
    scroll_state: RefCell<ScrollState>,
    is_generating: bool,
    partial_response: String,
    selected_model: usize,
    system_prompt: String,
    total_tokens: usize,
    show_sidebar: bool,
    send_btn_rect: RefCell<Rect>,
    input_rect: RefCell<Rect>,
    model_btn_rects: RefCell<Vec<Rect>>,
    sidebar_toggle_rect: RefCell<Rect>,
}

impl App {
    fn new() -> Self {
        let mut input = TextInputState::new();
        input.focused = true;
        Self {
            messages: vec![ChatMessage {
                role: Role::System,
                content: "You are a helpful AI assistant. Be concise and clear.".into(),
                model_name: None,
                token_count: 0,
            }],
            input_state: RefCell::new(input),
            scroll_state: RefCell::new(ScrollState::new()),
            is_generating: false,
            partial_response: String::new(),
            selected_model: 1, // Default to Claude Opus 4
            system_prompt: "You are a helpful AI assistant. Be concise and clear.".into(),
            total_tokens: 0,
            show_sidebar: true,
            send_btn_rect: RefCell::new(Rect::ZERO),
            input_rect: RefCell::new(Rect::ZERO),
            model_btn_rects: RefCell::new(Vec::new()),
            sidebar_toggle_rect: RefCell::new(Rect::ZERO),
        }
    }

    fn current_model(&self) -> &LlmModel {
        &MODELS[self.selected_model]
    }

    /// Estimate token count (rough: ~4 chars per token).
    fn estimate_tokens(text: &str) -> usize {
        text.len().div_ceil(4)
    }
}

// ── Messages ────────────────────────────────────────────────────────

#[derive(Debug)]
enum Msg {
    SendMessage,
    AppendToken(String),
    FinishResponse,
    SelectModel(usize),
    ToggleSidebar,
    ClearHistory,
}

// ── Simulated LLM responses ────────────────────────────────────────

const RESPONSES: &[&str] = &[
    "I'd be happy to help with that! Here's what I think:\n\n\
     The key to building great AI applications is choosing the right \
     architecture from the start. A clean separation between your model \
     layer, your business logic, and your presentation layer will pay \
     dividends as the project grows.\n\n\
     For Rust specifically, the Elm architecture pattern works \
     exceptionally well because it leverages the type system to make \
     invalid states unrepresentable.",
    "That's a great question! Let me break it down:\n\n\
     **1. Context window management** — Modern LLMs have large context \
     windows (128K-10M tokens), but filling them isn't free. Be strategic \
     about what you send.\n\n\
     **2. Prompt engineering** — Clear, structured prompts consistently \
     outperform vague ones. Use system messages to set behavior.\n\n\
     **3. Streaming** — Always stream responses in production. Users \
     perceive streaming as faster even when total latency is the same.",
    "Absolutely! Here's my recommended approach:\n\n\
     ```rust\n\
     // Define your state clearly\n\
     struct ChatState {\n\
         messages: Vec<Message>,\n\
         model: ModelConfig,\n\
         tokens_used: usize,\n\
     }\n\
     ```\n\n\
     The beauty of this pattern is that your entire chat state is serializable, \
     inspectable, and trivially testable. No hidden mutable state, no race \
     conditions, no surprises.\n\n\
     Want me to elaborate on any of these patterns?",
    "Here's how I'd think about that problem:\n\n\
     **Latency optimization:**\n\
     - Use streaming to show first tokens immediately\n\
     - Pre-warm connections to avoid cold starts\n\
     - Cache common responses where appropriate\n\n\
     **Cost optimization:**\n\
     - Choose the right model for each task (don't use GPT-4o for simple classification)\n\
     - Implement token budgets per conversation\n\
     - Use smaller models for summarization and routing\n\n\
     The key insight is that production LLM systems are rarely one model — \
     they're orchestrated pipelines of multiple models working together.",
    "I appreciate you asking about that! Security in LLM applications \
     is critically important. Here are the essentials:\n\n\
     1. **Never trust LLM output** — always validate and sanitize\n\
     2. **Implement rate limiting** — both per-user and global\n\
     3. **Use structured outputs** — JSON schemas prevent injection\n\
     4. **Audit everything** — log prompts, responses, and tool calls\n\
     5. **Least privilege** — LLM tool access should be minimal\n\n\
     Remember: an LLM is an untrusted input source, just like user input.",
];

fn pick_response(index: usize) -> &'static str {
    RESPONSES[index % RESPONSES.len()]
}

// ── Model implementation ────────────────────────────────────────────

impl Model for App {
    type Msg = Msg;

    fn update(&mut self, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::SendMessage => {
                let text = self.input_state.borrow().text.trim().to_string();
                if text.is_empty() || self.is_generating {
                    return Command::None;
                }

                let user_tokens = App::estimate_tokens(&text);
                self.total_tokens += user_tokens;

                self.messages.push(ChatMessage {
                    role: Role::User,
                    content: text,
                    model_name: None,
                    token_count: user_tokens,
                });

                // Clear input, keep focus
                let mut new_state = TextInputState::new();
                new_state.focused = true;
                *self.input_state.borrow_mut() = new_state;

                self.is_generating = true;
                self.partial_response.clear();

                // Simulate streaming word-by-word
                let full_response = pick_response(self.messages.len()).to_string();
                let words: Vec<String> = full_response
                    .split_inclusive(char::is_whitespace)
                    .map(String::from)
                    .collect();

                let mut cmds: Vec<Command<Msg>> = Vec::new();
                let mut accumulated = String::new();
                for word in &words {
                    accumulated.push_str(word);
                    let snapshot = accumulated.clone();
                    cmds.push(Command::Task(Box::new(move || {
                        std::thread::sleep(Duration::from_millis(25));
                        Msg::AppendToken(snapshot)
                    })));
                }
                cmds.push(Command::Task(Box::new(|| {
                    std::thread::sleep(Duration::from_millis(25));
                    Msg::FinishResponse
                })));

                Command::Batch(cmds)
            }

            Msg::AppendToken(text) => {
                self.partial_response = text;
                self.scroll_state.borrow_mut().offset_y += 18.0;
                Command::None
            }

            Msg::FinishResponse => {
                if !self.partial_response.is_empty() {
                    let resp_tokens = App::estimate_tokens(&self.partial_response);
                    self.total_tokens += resp_tokens;
                    self.messages.push(ChatMessage {
                        role: Role::Assistant,
                        content: self.partial_response.clone(),
                        model_name: Some(self.current_model().name.to_string()),
                        token_count: resp_tokens,
                    });
                }
                self.partial_response.clear();
                self.is_generating = false;
                self.scroll_state.borrow_mut().offset_y += 80.0;
                Command::None
            }

            Msg::SelectModel(index) => {
                if index < MODELS.len() {
                    self.selected_model = index;
                }
                Command::None
            }

            Msg::ToggleSidebar => {
                self.show_sidebar = !self.show_sidebar;
                Command::None
            }

            Msg::ClearHistory => {
                self.messages.clear();
                self.messages.push(ChatMessage {
                    role: Role::System,
                    content: self.system_prompt.clone(),
                    model_name: None,
                    token_count: 0,
                });
                self.total_tokens = 0;
                self.scroll_state.borrow_mut().offset_y = 0.0;
                Command::None
            }
        }
    }

    fn view(&self, frame: &mut Frame<'_>) {
        let area = frame.area;

        // ┌──────────┬──────────────────────────┐
        // │ Sidebar  │  Title bar               │ 36px
        // │          ├──────────────────────────┤
        // │ Models   │                          │
        // │          │  Chat messages (scroll)  │ fill
        // │ Stats    │                          │
        // │          ├──────────────────────────┤
        // │          │  [Input           ][Send]│ 44px
        // └──────────┴──────────────────────────┘

        let columns = if self.show_sidebar {
            Layout::new(
                Direction::Horizontal,
                [Constraint::Length(200.0), Constraint::Fill(1.0)],
            )
            .split(area)
        } else {
            Layout::new(
                Direction::Horizontal,
                [Constraint::Length(0.0), Constraint::Fill(1.0)],
            )
            .split(area)
        };

        // ── Sidebar ────────────────────────────────────────────────
        if self.show_sidebar {
            self.render_sidebar(columns[0], frame);
        }

        // ── Main content ───────────────────────────────────────────
        let main_rows = Layout::new(
            Direction::Vertical,
            [
                Constraint::Length(36.0),
                Constraint::Fill(1.0),
                Constraint::Length(44.0),
            ],
        )
        .split(columns[1]);

        self.render_title_bar(main_rows[0], frame);
        self.render_messages(main_rows[1], frame);
        self.render_input_bar(main_rows[2], frame);
    }

    fn handle_event(&self, event: Event) -> Option<Msg> {
        match event {
            // Text input → insert into buffer
            Event::TextInput(ref text) => {
                if !self.is_generating {
                    let mut state = self.input_state.borrow_mut();
                    let cursor = state.cursor;
                    state.text.insert_str(cursor, text);
                    state.cursor = cursor + text.len();
                    state.focused = true;
                }
                None
            }

            // Enter → send message
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                kind: KeyEventKind::Press,
                modifiers,
            }) if modifiers.is_empty() => Some(Msg::SendMessage),

            // Ctrl+L → clear history
            Event::Key(KeyEvent {
                code: KeyCode::Char('l'),
                kind: KeyEventKind::Press,
                modifiers,
            }) if modifiers.contains(KeyModifiers::CONTROL) => Some(Msg::ClearHistory),

            // Ctrl+B → toggle sidebar
            Event::Key(KeyEvent {
                code: KeyCode::Char('b'),
                kind: KeyEventKind::Press,
                modifiers,
            }) if modifiers.contains(KeyModifiers::CONTROL) => Some(Msg::ToggleSidebar),

            // Backspace
            Event::Key(KeyEvent {
                code: KeyCode::Backspace,
                kind: KeyEventKind::Press,
                ..
            }) => {
                let mut state = self.input_state.borrow_mut();
                if state.cursor > 0 {
                    let new_cursor = state.text[..state.cursor]
                        .char_indices()
                        .next_back()
                        .map(|(i, _)| i)
                        .unwrap_or(0);
                    let old_cursor = state.cursor;
                    state.text.drain(new_cursor..old_cursor);
                    state.cursor = new_cursor;
                }
                None
            }

            // Delete
            Event::Key(KeyEvent {
                code: KeyCode::Delete,
                kind: KeyEventKind::Press,
                ..
            }) => {
                let mut state = self.input_state.borrow_mut();
                let cursor = state.cursor;
                if cursor < state.text.len() {
                    let next = state.text[cursor..]
                        .char_indices()
                        .nth(1)
                        .map(|(i, _)| cursor + i)
                        .unwrap_or(state.text.len());
                    state.text.drain(cursor..next);
                }
                None
            }

            // Arrow keys
            Event::Key(KeyEvent {
                code: KeyCode::Left,
                kind: KeyEventKind::Press,
                ..
            }) => {
                let mut state = self.input_state.borrow_mut();
                if state.cursor > 0 {
                    state.cursor = state.text[..state.cursor]
                        .char_indices()
                        .next_back()
                        .map(|(i, _)| i)
                        .unwrap_or(0);
                }
                None
            }
            Event::Key(KeyEvent {
                code: KeyCode::Right,
                kind: KeyEventKind::Press,
                ..
            }) => {
                let mut state = self.input_state.borrow_mut();
                if state.cursor < state.text.len() {
                    state.cursor = state.text[state.cursor..]
                        .char_indices()
                        .nth(1)
                        .map(|(i, _)| state.cursor + i)
                        .unwrap_or(state.text.len());
                }
                None
            }

            // Home / End
            Event::Key(KeyEvent {
                code: KeyCode::Home,
                kind: KeyEventKind::Press,
                ..
            }) => {
                self.input_state.borrow_mut().cursor = 0;
                None
            }
            Event::Key(KeyEvent {
                code: KeyCode::End,
                kind: KeyEventKind::Press,
                ..
            }) => {
                let mut state = self.input_state.borrow_mut();
                state.cursor = state.text.len();
                None
            }

            // Mouse click — check send button, sidebar model buttons, sidebar toggle
            Event::Mouse(ref mouse) if mouse.is_click() => {
                // Send button
                let btn_rect = *self.send_btn_rect.borrow();
                if btn_rect.contains(mouse.position) && !self.is_generating {
                    return Some(Msg::SendMessage);
                }

                // Model buttons
                let rects = self.model_btn_rects.borrow();
                for (i, rect) in rects.iter().enumerate() {
                    if rect.contains(mouse.position) {
                        return Some(Msg::SelectModel(i));
                    }
                }

                // Sidebar toggle
                let toggle_rect = *self.sidebar_toggle_rect.borrow();
                if toggle_rect.contains(mouse.position) {
                    return Some(Msg::ToggleSidebar);
                }

                // Focus input
                let input_rect = *self.input_rect.borrow();
                if input_rect.contains(mouse.position) {
                    self.input_state.borrow_mut().focused = true;
                }
                None
            }

            _ => None,
        }
    }

    fn register_ontology(&self, registry: &mut OntologyRegistry) {
        registry.register_schema(WidgetSchema::new(
            "LlmChatApp",
            "LLM chat interface with model selection, streaming responses, and token tracking",
            SemanticRole::Container,
        ));
    }

    fn title(&self) -> &str {
        "Dewey + agpu — LLM Chat"
    }
}

// ── View helpers ────────────────────────────────────────────────────

impl App {
    fn render_sidebar(&self, area: Rect, frame: &mut Frame<'_>) {
        // Sidebar background
        Container::new()
            .agent_id("sidebar")
            .style(Style::new().bg(Color::from_rgb8(24, 24, 38)))
            .render(area, frame);

        let sidebar = area.inner(&Margin::uniform(8.0));
        let line_h = 22.0;

        // Title
        Label::new("Models")
            .agent_id("sidebar_title")
            .style(Style::new().fg(Color::from_rgb8(180, 180, 220)))
            .render(
                Rect::new(sidebar.x, sidebar.y, sidebar.width, line_h + 4.0),
                frame,
            );

        // Model list
        let mut rects = Vec::new();
        let model_start_y = sidebar.y + line_h + 12.0;
        for (i, model) in MODELS.iter().enumerate() {
            let is_selected = i == self.selected_model;
            let y = model_start_y + (i as f32) * (line_h * 2.0 + 8.0);

            let btn_rect = Rect::new(sidebar.x, y, sidebar.width, line_h * 2.0 + 4.0);
            rects.push(btn_rect);

            // Model name
            let name_color = if is_selected {
                Color::from_rgb8(130, 200, 255)
            } else {
                Color::from_rgb8(200, 200, 210)
            };
            let prefix = if is_selected { "> " } else { "  " };
            Label::new(format!("{}{}", prefix, model.name))
                .agent_id(format!("model_{}", model.id))
                .style(Style::new().fg(name_color))
                .render(Rect::new(sidebar.x, y, sidebar.width, line_h), frame);

            // Provider + context
            Label::new(format!("  {} | {}", model.provider, model.context_window))
                .agent_id(format!("model_{}_info", model.id))
                .style(Style::new().fg(Color::from_rgb8(120, 120, 140)))
                .render(
                    Rect::new(sidebar.x, y + line_h, sidebar.width, line_h),
                    frame,
                );
        }
        *self.model_btn_rects.borrow_mut() = rects;

        // Stats at bottom
        let stats_y = sidebar.y + sidebar.height - line_h * 4.0;

        Label::new("───────────")
            .agent_id("sidebar_divider")
            .style(Style::new().fg(Color::from_rgb8(60, 60, 80)))
            .render(Rect::new(sidebar.x, stats_y, sidebar.width, line_h), frame);

        Label::new(format!("Messages: {}", self.messages.len()))
            .agent_id("stat_messages")
            .style(Style::new().fg(Color::from_rgb8(160, 160, 180)))
            .render(
                Rect::new(sidebar.x, stats_y + line_h, sidebar.width, line_h),
                frame,
            );

        Label::new(format!("Tokens: ~{}", self.total_tokens))
            .agent_id("stat_tokens")
            .style(Style::new().fg(Color::from_rgb8(160, 160, 180)))
            .render(
                Rect::new(sidebar.x, stats_y + line_h * 2.0, sidebar.width, line_h),
                frame,
            );

        Label::new("Ctrl+L clear | Ctrl+B hide")
            .agent_id("sidebar_shortcuts")
            .style(Style::new().fg(Color::from_rgb8(100, 100, 120)))
            .render(
                Rect::new(sidebar.x, stats_y + line_h * 3.0, sidebar.width, line_h),
                frame,
            );
    }

    fn render_title_bar(&self, area: Rect, frame: &mut Frame<'_>) {
        Container::new()
            .agent_id("title_bar")
            .style(Style::new().bg(Color::from_rgb8(30, 30, 46)))
            .render(area, frame);

        let cols = Layout::new(
            Direction::Horizontal,
            [
                Constraint::Length(36.0),
                Constraint::Fill(1.0),
                Constraint::Length(160.0),
            ],
        )
        .split(area);

        // Sidebar toggle
        let toggle_label = if self.show_sidebar { "<<" } else { ">>" };
        *self.sidebar_toggle_rect.borrow_mut() = cols[0];
        Label::new(toggle_label)
            .agent_id("sidebar_toggle")
            .style(Style::new().fg(Color::from_rgb8(140, 140, 160)))
            .render(cols[0], frame);

        // Title
        Label::new(format!("  LLM Chat — {}", self.current_model().name))
            .agent_id("chat_title")
            .style(Style::new().fg(Color::from_rgb8(220, 220, 240)))
            .render(cols[1], frame);

        // Status
        let status = if self.is_generating {
            format!("Generating... | ~{} tokens", self.total_tokens)
        } else {
            format!("Ready | ~{} tokens", self.total_tokens)
        };
        Label::new(status)
            .agent_id("chat_status")
            .style(Style::new().fg(Color::from_rgb8(130, 130, 160)))
            .render(cols[2], frame);
    }

    fn render_messages(&self, area: Rect, frame: &mut Frame<'_>) {
        let msg_area = area.inner(&Margin::new(4.0, 8.0, 4.0, 8.0));
        let line_h = 20.0;

        // Scroll area
        ScrollArea::vertical().agent_id("chat_scroll").render(
            msg_area,
            frame,
            &mut self.scroll_state.borrow_mut(),
        );

        // Render messages
        let mut y_offset = 0.0;
        for (i, msg) in self.messages.iter().enumerate() {
            let (prefix, header_color, content_color) = match msg.role {
                Role::User => (
                    "You",
                    Color::from_rgb8(130, 180, 255),
                    Color::from_rgb8(210, 220, 240),
                ),
                Role::Assistant => (
                    msg.model_name.as_deref().unwrap_or("Assistant"),
                    Color::from_rgb8(160, 230, 160),
                    Color::from_rgb8(210, 230, 210),
                ),
                Role::System => (
                    "System",
                    Color::from_rgb8(200, 200, 130),
                    Color::from_rgb8(190, 190, 160),
                ),
            };

            // Role header
            let token_info = if msg.token_count > 0 {
                format!("  ({} tokens)", msg.token_count)
            } else {
                String::new()
            };
            let header_rect = Rect::new(msg_area.x, msg_area.y + y_offset, msg_area.width, line_h);
            Label::new(format!("━━ {} ━━{}", prefix, token_info))
                .agent_id(format!("msg_{}_header", i))
                .style(Style::new().fg(header_color))
                .render(header_rect, frame);
            y_offset += line_h;

            // Message content — estimate line wrapping
            let chars_per_line = ((msg_area.width - 24.0) / 8.0).max(40.0);
            let content_lines = (msg.content.len() as f32 / chars_per_line).ceil().max(1.0);
            let content_height = content_lines * line_h;
            let content_rect = Rect::new(
                msg_area.x + 12.0,
                msg_area.y + y_offset,
                msg_area.width - 24.0,
                content_height,
            );
            Label::new(&msg.content)
                .agent_id(format!("msg_{}_content", i))
                .style(Style::new().fg(content_color))
                .render(content_rect, frame);
            y_offset += content_height + 12.0;
        }

        // Show streaming partial response
        if self.is_generating && !self.partial_response.is_empty() {
            let model_name = self.current_model().name;
            let header_rect = Rect::new(msg_area.x, msg_area.y + y_offset, msg_area.width, line_h);
            Label::new(format!("━━ {} ━━  ...", model_name))
                .agent_id("msg_streaming_header")
                .style(Style::new().fg(Color::from_rgb8(160, 230, 160)))
                .render(header_rect, frame);
            y_offset += line_h;

            let chars_per_line = ((msg_area.width - 24.0) / 8.0).max(40.0);
            let content_lines = (self.partial_response.len() as f32 / chars_per_line)
                .ceil()
                .max(1.0);
            let content_height = content_lines * line_h;
            let content_rect = Rect::new(
                msg_area.x + 12.0,
                msg_area.y + y_offset,
                msg_area.width - 24.0,
                content_height,
            );
            Label::new(&self.partial_response)
                .agent_id("msg_streaming_content")
                .style(Style::new().fg(Color::from_rgb8(180, 220, 180)))
                .render(content_rect, frame);
        }
    }

    fn render_input_bar(&self, area: Rect, frame: &mut Frame<'_>) {
        let cols = Layout::new(
            Direction::Horizontal,
            [Constraint::Fill(1.0), Constraint::Length(90.0)],
        )
        .split(area);

        *self.input_rect.borrow_mut() = cols[0];
        *self.send_btn_rect.borrow_mut() = cols[1];

        let placeholder = if self.is_generating {
            "Waiting for response..."
        } else {
            "Message... (Enter to send)"
        };
        TextInput::new()
            .placeholder(placeholder)
            .agent_id("chat_input")
            .render(cols[0], frame, &mut self.input_state.borrow_mut());

        let btn_label = if self.is_generating { "..." } else { "Send" };
        Button::new(btn_label)
            .agent_id("send_btn")
            .enabled(!self.is_generating)
            .render(cols[1], frame);
    }
}

// ── Main ────────────────────────────────────────────────────────────

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    AgpuProgram::new(App::new())
        .with_options(ProgramOptions {
            width: 900.0,
            height: 600.0,
            ..Default::default()
        })
        .with_profiling(true)
        .run()
}
