//! Hello World — minimal Dewey application.

use dewey::prelude::*;

struct App;

enum Msg {}

impl Model for App {
    type Msg = Msg;

    fn update(&mut self, msg: Msg) -> Command<Msg> {
        match msg {}
    }

    fn view(&self, frame: &mut Frame<'_>) {
        let area = frame.area;
        Label::new("Hello, Dewey!")
            .agent_id("hello_label")
            .render(area, frame);
    }

    fn handle_event(&self, event: Event) -> Option<Msg> {
        if let Event::CloseRequested = event {
            // Window close handled by runtime
        }
        None
    }

    fn title(&self) -> &str {
        "Dewey — Hello World"
    }
}

fn main() -> std::result::Result<(), eframe::Error> {
    env_logger::init();
    Program::new(App)
        .with_options(ProgramOptions {
            width: 400.0,
            height: 300.0,
            ..Default::default()
        })
        .run()
}
