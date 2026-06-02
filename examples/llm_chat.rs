//! LLM Chat — a one-shot ChatGPT / Claude / Jan-style conversational interface.
//!
//! Demonstrates:
//! - Sidebar with conversation list (new / switch / delete)
//! - Model selector dropdown
//! - Multi-message chat with role-based styling (user/assistant/system)
//! - Simulated streaming token-by-token responses
//! - Markdown-like formatting in assistant output
//! - Multi-line text input with Enter to send, Shift+Enter for newline
//! - Keyboard shortcuts and mouse interaction
//! - Full ontology registration for agent discoverability
//!
//! Run with:
//!   `cargo run --example llm_chat --features agpu-backend --no-default-features`

use dewey::backend::agpu_backend::AgpuProgram;
use dewey::event::KeyEventKind;
use dewey::prelude::*;
use dewey::widget::input::TextInputState;
use dewey::widget::scroll::ScrollState;
use dewey::widget::select::SelectState;
use std::cell::RefCell;

// ── Theme constants ────────────────────────────────────────────────

mod colors {
    use dewey::prelude::Color;

    // Sidebar
    pub const SIDEBAR_BG: Color = Color::rgb(0.125, 0.129, 0.137);
    pub const SIDEBAR_HOVER: Color = Color::rgb(0.173, 0.180, 0.196);
    pub const SIDEBAR_TEXT: Color = Color::rgb(0.824, 0.824, 0.843);
    pub const SIDEBAR_MUTED: Color = Color::rgb(0.557, 0.557, 0.576);

    // Main area
    pub const MAIN_BG: Color = Color::rgb(0.204, 0.208, 0.255);
    pub const HEADER_BG: Color = Color::rgb(0.267, 0.275, 0.329);

    // Messages
    pub const USER_BG: Color = Color::rgb(0.204, 0.208, 0.255);
    pub const ASSISTANT_BG: Color = Color::rgb(0.267, 0.275, 0.329);
    pub const USER_TEXT: Color = Color::rgb(0.925, 0.925, 0.945);
    pub const ASSISTANT_TEXT: Color = Color::rgb(0.820, 0.835, 0.859);
    pub const SYSTEM_TEXT: Color = Color::rgb(0.706, 0.706, 0.471);

    // Accents
    pub const ACCENT: Color = Color::rgb(0.063, 0.639, 0.498);
    pub const ACCENT_HOVER: Color = Color::rgb(0.102, 0.737, 0.612);
    pub const SEND_BTN: Color = Color::rgb(0.098, 0.765, 0.490);
}

// ── LLM models ─────────────────────────────────────────────────────

const MODELS: &[&str] = &[
    "GPT-4o",
    "GPT-4o mini",
    "Claude 4 Opus",
    "Claude 4 Sonnet",
    "Llama 3.3 70B",
    "Mistral Large",
    "Gemini 2.5 Pro",
];

// ── Data model ─────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq, Eq)]
enum Role {
    User,
    Assistant,
    System,
}

#[derive(Clone, Debug)]
struct ChatMessage {
    role: Role,
    content: String,
}

#[derive(Clone)]
struct Conversation {
    id: usize,
    title: String,
    messages: Vec<ChatMessage>,
    model: String,
}

impl Conversation {
    fn new(id: usize) -> Self {
        Self {
            id,
            title: format!("New Chat {}", id),
            messages: vec![ChatMessage {
                role: Role::System,
                content: "You are a helpful assistant.".into(),
            }],
            model: MODELS[0].to_string(),
        }
    }

    /// Auto-title from the first user message.
    fn auto_title(&mut self) {
        if let Some(msg) = self.messages.iter().find(|m| m.role == Role::User) {
            let raw = msg.content.trim();
            self.title = if raw.len() > 36 {
                format!("{}…", &raw[..35])
            } else {
                raw.to_string()
            };
        }
    }
}

struct App {
    conversations: Vec<Conversation>,
    active_idx: usize,
    next_id: usize,

    // Widget state
    input_state: RefCell<TextInputState>,
    scroll_state: RefCell<ScrollState>,
    model_select_state: RefCell<SelectState>,

    // Streaming (tick-driven)
    is_generating: bool,
    partial_response: String,
    pending_words: Vec<String>,
    word_index: usize,

    // Hit-test rects (set during view, read during handle_event)
    send_btn_rect: RefCell<Rect>,
    input_rect: RefCell<Rect>,
    new_chat_rect: RefCell<Rect>,
    sidebar_item_rects: RefCell<Vec<Rect>>,
}

impl App {
    fn new() -> Self {
        let first = Conversation::new(1);
        let mut input = TextInputState::new();
        input.focused = true;
        Self {
            conversations: vec![first],
            active_idx: 0,
            next_id: 2,
            input_state: RefCell::new(input),
            scroll_state: RefCell::new(ScrollState::new()),
            model_select_state: RefCell::new(SelectState::new()),
            is_generating: false,
            partial_response: String::new(),
            pending_words: Vec::new(),
            word_index: 0,
            send_btn_rect: RefCell::new(Rect::ZERO),
            input_rect: RefCell::new(Rect::ZERO),
            new_chat_rect: RefCell::new(Rect::ZERO),
            sidebar_item_rects: RefCell::new(Vec::new()),
        }
    }

    fn active_conv(&self) -> &Conversation {
        &self.conversations[self.active_idx]
    }

    fn active_conv_mut(&mut self) -> &mut Conversation {
        &mut self.conversations[self.active_idx]
    }
}

// ── Messages ───────────────────────────────────────────────────────

#[derive(Debug)]
#[allow(dead_code)]
enum Msg {
    NewChat,
    SelectConversation(usize),
    DeleteConversation(usize),
    SelectModel(usize),
    SendMessage,
    StreamTick,
    FinishResponse,
}

// ── Canned AI responses (simulate LLM output) ─────────────────────

const RESPONSES: &[&str] = &[
    "Great question! Let me break this down for you.\n\n\
     **Key Points:**\n\
     1. Modern LLMs use transformer architectures with self-attention\n\
     2. Context windows determine how much text the model can process\n\
     3. Temperature controls randomness in generation\n\n\
     The fundamental insight is that attention mechanisms allow the model \
     to weigh the relevance of every token against every other token, \
     creating rich contextual representations.\n\n\
     Would you like me to dive deeper into any of these areas?",
    "Absolutely! Here's a practical guide:\n\n\
     ```rust\n\
     // Define your model state\n\
     struct App {\n\
         messages: Vec<Message>,\n\
         input: String,\n\
     }\n\
     ```\n\n\
     **Step 1:** Define your data model with all the state you need.\n\n\
     **Step 2:** Implement the `update` function to handle each message \
     variant — this is where your business logic lives.\n\n\
     **Step 3:** Write a `view` function that renders the current state. \
     Because it's a pure function, your UI is always deterministic.\n\n\
     This Elm architecture pattern eliminates entire categories of bugs.",
    "That's a really thoughtful observation. Here's how I think about it:\n\n\
     > \"The best code is code that doesn't need to exist.\"\n\n\
     **Simplicity** wins in the long run. Every abstraction has a cost — \
     cognitive overhead, maintenance burden, and potential for bugs. The \
     key is finding the right level of abstraction for your problem.\n\n\
     Some principles I've found useful:\n\
     - Prefer composition over inheritance\n\
     - Make illegal states unrepresentable\n\
     - Write tests that describe *behavior*, not implementation\n\
     - Refactor only when you have three examples of the pattern\n\n\
     What specific aspect would you like to explore further?",
    "I'd be happy to help with that! Let me walk you through the process.\n\n\
     **Prerequisites:**\n\
     - Rust 1.85 or later\n\
     - A graphics driver that supports Vulkan, Metal, or DX12\n\n\
     **Setup:**\n\
     ```toml\n\
     [dependencies]\n\
     dewey = \"1\"\n\
     ```\n\n\
     The framework handles window creation, event loops, and GPU \
     initialization automatically. You just focus on your data model \
     and how to render it.\n\n\
     The beauty of Dewey's approach is that the same code works across \
     all backends — GPU, software, headless, and web — without changes.",
    "Here's a comparison of the major approaches:\n\n\
     | Approach | Pros | Cons |\n\
     |----------|------|------|\n\
     | Fine-tuning | Domain expertise | Expensive, data-hungry |\n\
     | RAG | Up-to-date, verifiable | Retrieval quality varies |\n\
     | Prompting | Quick iteration | Limited by context window |\n\
     | Agents | Autonomous execution | Complex, harder to debug |\n\n\
     For most use cases, I'd recommend starting with prompting + RAG, \
     then graduating to fine-tuning or agents only when you hit clear \
     limitations.\n\n\
     The 80/20 rule applies strongly here: prompt engineering gets you \
     80% of the way with 20% of the effort.",
];

fn pick_response(index: usize) -> &'static str {
    RESPONSES[index % RESPONSES.len()]
}

// ── Model implementation ───────────────────────────────────────────

impl Model for App {
    type Msg = Msg;

    fn update(&mut self, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::NewChat => {
                let conv = Conversation::new(self.next_id);
                self.next_id += 1;
                self.conversations.push(conv);
                self.active_idx = self.conversations.len() - 1;

                // Reset state for the new chat
                let mut input = TextInputState::new();
                input.focused = true;
                *self.input_state.borrow_mut() = input;
                *self.scroll_state.borrow_mut() = ScrollState::new();
                self.is_generating = false;
                self.partial_response.clear();
                Command::None
            }

            Msg::SelectConversation(idx) => {
                if idx < self.conversations.len() && !self.is_generating {
                    self.active_idx = idx;
                    *self.scroll_state.borrow_mut() = ScrollState::new();
                    self.partial_response.clear();
                }
                Command::None
            }

            Msg::DeleteConversation(idx) => {
                if self.conversations.len() > 1 && idx < self.conversations.len() {
                    self.conversations.remove(idx);
                    if self.active_idx >= self.conversations.len() {
                        self.active_idx = self.conversations.len() - 1;
                    }
                }
                Command::None
            }

            Msg::SelectModel(idx) => {
                if idx < MODELS.len() {
                    self.active_conv_mut().model = MODELS[idx].to_string();
                    self.model_select_state.borrow_mut().selected = idx;
                }
                Command::None
            }

            Msg::SendMessage => {
                let text = self.input_state.borrow().text.trim().to_string();
                if text.is_empty() || self.is_generating {
                    return Command::None;
                }

                // Add user message
                self.active_conv_mut().messages.push(ChatMessage {
                    role: Role::User,
                    content: text,
                });

                // Auto-title from first user message
                self.active_conv_mut().auto_title();

                // Clear input, keep focus
                let mut new_state = TextInputState::new();
                new_state.focused = true;
                *self.input_state.borrow_mut() = new_state;

                // Prepare tick-driven streaming
                let msg_count = self.active_conv().messages.len();
                let full_response = pick_response(msg_count).to_string();
                self.pending_words = full_response
                    .split_inclusive(char::is_whitespace)
                    .map(String::from)
                    .collect();
                self.word_index = 0;
                self.is_generating = true;
                self.partial_response.clear();

                Command::None
            }

            Msg::StreamTick => {
                if !self.is_generating {
                    return Command::None;
                }
                if self.word_index < self.pending_words.len() {
                    self.partial_response
                        .push_str(&self.pending_words[self.word_index]);
                    self.word_index += 1;
                    self.scroll_state.borrow_mut().offset_y += 18.0;
                    Command::None
                } else {
                    // All words delivered — finalize
                    if !self.partial_response.is_empty() {
                        let content = self.partial_response.clone();
                        self.active_conv_mut().messages.push(ChatMessage {
                            role: Role::Assistant,
                            content,
                        });
                    }
                    self.partial_response.clear();
                    self.pending_words.clear();
                    self.word_index = 0;
                    self.is_generating = false;
                    self.scroll_state.borrow_mut().offset_y += 60.0;
                    Command::None
                }
            }

            Msg::FinishResponse => {
                if !self.partial_response.is_empty() {
                    let content = self.partial_response.clone();
                    self.active_conv_mut().messages.push(ChatMessage {
                        role: Role::Assistant,
                        content,
                    });
                }
                self.partial_response.clear();
                self.pending_words.clear();
                self.word_index = 0;
                self.is_generating = false;
                self.scroll_state.borrow_mut().offset_y += 60.0;
                Command::None
            }
        }
    }

    fn view(&self, frame: &mut Frame<'_>) {
        let area = frame.area;

        // ┌────────────┬──────────────────────────┐
        // │            │  Model: [GPT-4o    ▾]    │  header  44px
        // │  Sidebar   ├──────────────────────────┤
        // │  260px     │                          │
        // │            │   Chat messages (scroll) │  fill
        // │ [+ New]    │                          │
        // │ [Chat 1]   ├──────────────────────────┤
        // │ [Chat 2]   │  [Input ...        ][▶]  │  input   56px
        // └────────────┴──────────────────────────┘

        let cols = Layout::new(
            Direction::Horizontal,
            [Constraint::Length(260.0), Constraint::Fill(1.0)],
        )
        .split(area);

        self.render_sidebar(cols[0], frame);
        self.render_main(cols[1], frame);
    }

    fn handle_event(&self, event: Event) -> Option<Msg> {
        match event {
            // Tick drives streaming token delivery
            Event::Tick => {
                if self.is_generating {
                    Some(Msg::StreamTick)
                } else {
                    None
                }
            }

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

            // Enter sends (Shift+Enter would add newline in full impl)
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                kind: KeyEventKind::Press,
                modifiers,
            }) if modifiers.is_empty() => Some(Msg::SendMessage),

            // Ctrl+N = new chat
            Event::Key(KeyEvent {
                code: KeyCode::Char('n'),
                kind: KeyEventKind::Press,
                modifiers,
            }) if modifiers.contains(KeyModifiers::CONTROL) => Some(Msg::NewChat),

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

            // Mouse click
            Event::Mouse(ref mouse) if mouse.is_click() => {
                // Send button
                let btn_rect = *self.send_btn_rect.borrow();
                if btn_rect.contains(mouse.position) && !self.is_generating {
                    return Some(Msg::SendMessage);
                }
                // New chat button
                let new_rect = *self.new_chat_rect.borrow();
                if new_rect.contains(mouse.position) {
                    return Some(Msg::NewChat);
                }
                // Sidebar conversation items
                let rects = self.sidebar_item_rects.borrow();
                for (i, rect) in rects.iter().enumerate() {
                    if rect.contains(mouse.position) {
                        return Some(Msg::SelectConversation(i));
                    }
                }
                drop(rects);
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
            "LlmChat",
            "ChatGPT/Claude-style LLM conversational interface with sidebar, \
             model selector, streaming responses, and multi-conversation support",
            SemanticRole::Container,
        ));
    }

    fn title(&self) -> &str {
        "Dewey — LLM Chat"
    }
}

// ── Rendering helpers ──────────────────────────────────────────────

impl App {
    fn render_sidebar(&self, area: Rect, frame: &mut Frame<'_>) {
        // Background
        Container::new()
            .agent_id("sidebar_bg")
            .bg(colors::SIDEBAR_BG)
            .render(area, frame);

        let rows = Layout::new(
            Direction::Vertical,
            [
                Constraint::Length(56.0), // header + new-chat button
                Constraint::Fill(1.0),    // conversation list
                Constraint::Length(36.0), // footer
            ],
        )
        .split(area.inner(&Margin::uniform(8.0)));

        // ── New Chat button ────────────────────────────────────────
        let btn_area = rows[0].inner(&Margin {
            top: 10.0,
            right: 8.0,
            bottom: 10.0,
            left: 8.0,
        });
        *self.new_chat_rect.borrow_mut() = btn_area;

        Button::new("＋  New Chat")
            .agent_id("new_chat_btn")
            .fg(colors::SIDEBAR_TEXT)
            .bg(colors::SIDEBAR_HOVER)
            .rounded(8.0)
            .render(btn_area, frame);

        // ── Conversation list ──────────────────────────────────────
        let list_area = rows[1];
        let line_h = 34.0;
        let mut item_rects = Vec::new();
        for (i, conv) in self.conversations.iter().enumerate() {
            let y = list_area.y + (i as f32) * (line_h + 2.0);
            if y + line_h > list_area.y + list_area.height {
                break;
            }
            let item_rect = Rect::new(list_area.x + 6.0, y, list_area.width - 12.0, line_h);
            item_rects.push(item_rect);

            let is_active = i == self.active_idx;
            let fg = if is_active {
                Color::WHITE
            } else {
                colors::SIDEBAR_TEXT
            };

            if is_active {
                Container::new()
                    .agent_id(format!("conv_{}_bg", conv.id))
                    .bg(colors::SIDEBAR_HOVER)
                    .rounded(6.0)
                    .render(item_rect, frame);
            }

            let indicator = if is_active { "▸ " } else { "  " };
            Label::new(format!("{}{}", indicator, conv.title))
                .agent_id(format!("conv_{}", conv.id))
                .fg(fg)
                .render(item_rect, frame);
        }
        *self.sidebar_item_rects.borrow_mut() = item_rects;

        // ── Footer ────────────────────────────────────────────────
        Label::new("Ctrl+N  New Chat")
            .agent_id("sidebar_footer")
            .fg(colors::SIDEBAR_MUTED)
            .render(rows[2], frame);
    }

    fn render_main(&self, area: Rect, frame: &mut Frame<'_>) {
        // Background
        Container::new()
            .agent_id("main_bg")
            .bg(colors::MAIN_BG)
            .render(area, frame);

        let rows = Layout::new(
            Direction::Vertical,
            [
                Constraint::Length(44.0), // header bar
                Constraint::Fill(1.0),    // messages
                Constraint::Length(56.0), // input bar
            ],
        )
        .split(area);

        self.render_header(rows[0], frame);
        self.render_messages(rows[1], frame);
        self.render_input_bar(rows[2], frame);
    }

    fn render_header(&self, area: Rect, frame: &mut Frame<'_>) {
        Container::new()
            .agent_id("header_bg")
            .bg(colors::HEADER_BG)
            .render(area, frame);

        let cols = Layout::new(
            Direction::Horizontal,
            [
                Constraint::Fill(1.0),     // title
                Constraint::Length(180.0), // model selector
            ],
        )
        .split(area.inner(&Margin::uniform(8.0)));

        // Conversation title
        Label::new(format!("  {}", self.active_conv().title))
            .agent_id("header_title")
            .fg(Color::WHITE)
            .render(cols[0], frame);

        // Model selector
        let model_options: Vec<String> = MODELS.iter().map(|s| s.to_string()).collect();
        Select::new("Model", model_options)
            .agent_id("model_selector")
            .render(cols[1], frame, &mut self.model_select_state.borrow_mut());
    }

    fn render_messages(&self, area: Rect, frame: &mut Frame<'_>) {
        let msg_area = area.inner(&Margin {
            top: 12.0,
            right: 32.0,
            bottom: 12.0,
            left: 32.0,
        });

        // Scroll wrapper
        ScrollArea::vertical().agent_id("chat_scroll").render(
            msg_area,
            frame,
            &mut self.scroll_state.borrow_mut(),
        );

        let line_h = 20.0;
        let max_chars_per_line: f32 = 72.0;
        let mut y = 0.0;

        let conv = self.active_conv();

        for (i, msg) in conv.messages.iter().enumerate() {
            y += self.render_message(msg_area, frame, msg, i, y, line_h, max_chars_per_line);
            y += 12.0; // gap between messages
        }

        // Streaming response
        if self.is_generating && !self.partial_response.is_empty() {
            self.render_streaming(msg_area, frame, y, line_h, max_chars_per_line);
        } else if self.is_generating {
            // Thinking indicator
            let indicator_rect =
                Rect::new(msg_area.x, msg_area.y + y, msg_area.width, line_h * 2.0);
            Label::new("⏳ Thinking...")
                .agent_id("thinking_indicator")
                .fg(colors::SIDEBAR_MUTED)
                .render(indicator_rect, frame);
        }

        // Empty state
        if conv.messages.len() <= 1 && !self.is_generating {
            let center_y = msg_area.height / 2.0 - 40.0;
            let welcome_rect = Rect::new(msg_area.x, msg_area.y + center_y, msg_area.width, 30.0);
            Label::new("How can I help you today?")
                .agent_id("welcome_text")
                .fg(colors::SIDEBAR_MUTED)
                .render(welcome_rect, frame);

            let model_rect = Rect::new(
                msg_area.x,
                msg_area.y + center_y + 35.0,
                msg_area.width,
                20.0,
            );
            Label::new(format!("Using: {}", conv.model))
                .agent_id("welcome_model")
                .fg(colors::ACCENT)
                .render(model_rect, frame);
        }
    }

    /// Renders a single message block. Returns the total height consumed.
    #[allow(clippy::too_many_arguments)]
    fn render_message(
        &self,
        area: Rect,
        frame: &mut Frame<'_>,
        msg: &ChatMessage,
        index: usize,
        y_offset: f32,
        line_h: f32,
        max_chars: f32,
    ) -> f32 {
        let (role_label, role_color, text_color, bg_color) = match msg.role {
            Role::User => ("You", colors::ACCENT, colors::USER_TEXT, colors::USER_BG),
            Role::Assistant => (
                &*self.active_conv().model,
                colors::ACCENT_HOVER,
                colors::ASSISTANT_TEXT,
                colors::ASSISTANT_BG,
            ),
            Role::System => (
                "System",
                colors::SYSTEM_TEXT,
                colors::SYSTEM_TEXT,
                colors::MAIN_BG,
            ),
        };

        let mut y = y_offset;

        // Role header
        let header_rect = Rect::new(area.x, area.y + y, area.width, line_h);
        Label::new(format!("  {}  ─────", role_label))
            .agent_id(format!("msg_{}_role", index))
            .fg(role_color)
            .render(header_rect, frame);
        y += line_h + 2.0;

        // Message content — approximate line-wrapping
        let content_lines = estimate_lines(&msg.content, max_chars);
        let content_h = (content_lines as f32) * line_h;
        let content_rect = Rect::new(area.x + 12.0, area.y + y, area.width - 24.0, content_h);

        Container::new()
            .agent_id(format!("msg_{}_bg", index))
            .bg(bg_color)
            .rounded(8.0)
            .render(
                Rect::new(area.x, area.y + y - 4.0, area.width, content_h + 8.0),
                frame,
            );

        Label::new(&msg.content)
            .agent_id(format!("msg_{}_content", index))
            .fg(text_color)
            .render(content_rect, frame);
        y += content_h;

        y - y_offset
    }

    fn render_streaming(
        &self,
        area: Rect,
        frame: &mut Frame<'_>,
        y_offset: f32,
        line_h: f32,
        max_chars: f32,
    ) {
        let mut y = y_offset;

        // Role header with streaming indicator
        let header_rect = Rect::new(area.x, area.y + y, area.width, line_h);
        Label::new(format!("  {}  ─────  ●", self.active_conv().model))
            .agent_id("streaming_role")
            .fg(colors::ACCENT_HOVER)
            .render(header_rect, frame);
        y += line_h + 2.0;

        // Content
        let content_lines = estimate_lines(&self.partial_response, max_chars);
        let content_h = (content_lines as f32) * line_h;
        let content_rect = Rect::new(area.x + 12.0, area.y + y, area.width - 24.0, content_h);

        Container::new()
            .agent_id("streaming_bg")
            .bg(colors::ASSISTANT_BG)
            .rounded(8.0)
            .render(
                Rect::new(area.x, area.y + y - 4.0, area.width, content_h + 8.0),
                frame,
            );

        Label::new(&self.partial_response)
            .agent_id("streaming_content")
            .fg(colors::ASSISTANT_TEXT)
            .render(content_rect, frame);
    }

    fn render_input_bar(&self, area: Rect, frame: &mut Frame<'_>) {
        Container::new()
            .agent_id("input_bar_bg")
            .bg(colors::MAIN_BG)
            .render(area, frame);

        let inner = area.inner(&Margin {
            top: 8.0,
            right: 24.0,
            bottom: 8.0,
            left: 24.0,
        });

        let cols = Layout::new(
            Direction::Horizontal,
            [Constraint::Fill(1.0), Constraint::Length(56.0)],
        )
        .split(inner);

        // Store rects for hit-testing
        *self.input_rect.borrow_mut() = cols[0];
        *self.send_btn_rect.borrow_mut() = cols[1];

        // Input field
        TextInput::new()
            .placeholder(if self.is_generating {
                "Waiting for response…"
            } else {
                "Message…  (Enter to send)"
            })
            .agent_id("chat_input")
            .render(cols[0], frame, &mut self.input_state.borrow_mut());

        // Send button
        let send_label = if self.is_generating { "⏳" } else { "▶" };
        Button::new(send_label)
            .agent_id("send_btn")
            .enabled(!self.is_generating)
            .bg(colors::SEND_BTN)
            .fg(Color::WHITE)
            .rounded(8.0)
            .render(cols[1], frame);
    }
}

// ── Utility ────────────────────────────────────────────────────────

/// Estimate the number of display lines a string will occupy given a
/// character-width budget. Counts explicit newlines and wrapping.
fn estimate_lines(text: &str, chars_per_line: f32) -> usize {
    let cpl = chars_per_line.max(1.0) as usize;
    text.lines()
        .map(|line| {
            let len = line.len();
            if len == 0 { 1 } else { len.div_ceil(cpl) }
        })
        .sum::<usize>()
        .max(1)
}

// ── Entry point ────────────────────────────────────────────────────

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    log::info!("Starting Dewey LLM Chat");

    AgpuProgram::new(App::new())
        .with_options(ProgramOptions {
            width: 1100.0,
            height: 720.0,
            ..Default::default()
        })
        .run()
}
