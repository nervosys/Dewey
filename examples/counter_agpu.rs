//! Counter — Dewey counter using the agpu GPU backend.
//!
//! Run with:
//!   cargo run --example counter_agpu --features agpu-backend --no-default-features

use dewey::backend::agpu_backend::AgpuProgram;
use dewey::prelude::*;

struct App {
    count: i32,
}

#[derive(Debug)]
enum Msg {
    Increment,
    Decrement,
    Reset,
}

impl Model for App {
    type Msg = Msg;

    fn update(&mut self, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::Increment => self.count += 1,
            Msg::Decrement => self.count -= 1,
            Msg::Reset => self.count = 0,
        }
        Command::None
    }

    fn view(&self, frame: &mut Frame<'_>) {
        let area = frame.area;

        let chunks = Layout::new(
            Direction::Vertical,
            [Constraint::Length(40.0), Constraint::Length(40.0)],
        )
        .split(area);

        Label::new(format!("Count: {}", self.count))
            .agent_id("counter_label")
            .render(chunks[0], frame);

        let btn_chunks = Layout::new(
            Direction::Horizontal,
            [
                Constraint::Percentage(33.3),
                Constraint::Percentage(33.3),
                Constraint::Percentage(33.3),
            ],
        )
        .split(chunks[1]);

        Button::new("- Decrement")
            .agent_id("decrement_btn")
            .render(btn_chunks[0], frame);

        Button::new("Reset")
            .agent_id("reset_btn")
            .render(btn_chunks[1], frame);

        Button::new("+ Increment")
            .agent_id("increment_btn")
            .render(btn_chunks[2], frame);
    }

    fn handle_event(&self, event: Event) -> Option<Msg> {
        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Char('+'),
                ..
            }) => Some(Msg::Increment),
            Event::Key(KeyEvent {
                code: KeyCode::Char('-'),
                ..
            }) => Some(Msg::Decrement),
            Event::Key(KeyEvent {
                code: KeyCode::Char('0'),
                ..
            }) => Some(Msg::Reset),
            _ => None,
        }
    }

    fn title(&self) -> &str {
        "Dewey + agpu — Counter"
    }
}

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    AgpuProgram::new(App { count: 0 })
        .with_options(ProgramOptions {
            width: 400.0,
            height: 200.0,
            ..Default::default()
        })
        .run()
}
