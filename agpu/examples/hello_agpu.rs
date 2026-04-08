//! Hello agpu — minimal standalone application.
//!
//! Demonstrates the Elm-architecture Model/update/view loop with
//! direct painter-based rendering (no widget library needed).
//!
//! Run with: `cargo run --example hello_agpu`

use agpu::prelude::*;

struct App {
    count: i32,
}

#[derive(Debug)]
enum Msg {
    Increment,
    Decrement,
}

impl Model for App {
    type Msg = Msg;

    fn update(&mut self, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::Increment => self.count += 1,
            Msg::Decrement => self.count -= 1,
        }
        Command::None
    }

    fn view(&self, frame: &mut Frame<'_>) {
        let area = frame.area;
        let btn_y = area.y + 50.0;
        let dec_rect = Rect::new(area.x + 10.0, btn_y, 80.0, 36.0);
        let inc_rect = Rect::new(area.x + 100.0, btn_y, 80.0, 36.0);

        // Paint everything first
        {
            let style = TextStyle {
                font_size: 24.0,
                color: Color::WHITE,
                ..Default::default()
            };
            let p = frame.painter();

            // Background
            p.fill_rect(area, Color::rgba(0.1, 0.1, 0.12, 1.0), 0.0);

            // Counter label
            p.text(
                Position::new(area.x + 10.0, area.y + 8.0),
                &format!("Count: {}", self.count),
                &style,
            );

            // Decrement button
            p.fill_rect(dec_rect, Color::rgba(0.8, 0.2, 0.2, 1.0), 4.0);
            p.text(
                Position::new(dec_rect.x + 30.0, dec_rect.y + 6.0),
                "−",
                &style,
            );

            // Increment button
            p.fill_rect(inc_rect, Color::rgba(0.2, 0.6, 0.2, 1.0), 4.0);
            p.text(
                Position::new(inc_rect.x + 30.0, inc_rect.y + 6.0),
                "+",
                &style,
            );
        }

        // Register hitboxes for event routing
        frame.register_hitbox("decrement_btn", dec_rect, 0);
        frame.register_hitbox("increment_btn", inc_rect, 0);
    }

    fn handle_event(&self, event: Event) -> Option<Msg> {
        if let Event::Mouse(me) = &event {
            if let agpu::MouseEventKind::Click(_) = me.kind {
                // Hit-testing handled by the runtime; here we just
                // demonstrate manual event mapping.
                if me.position.y > 50.0 {
                    return if me.position.x < 95.0 {
                        Some(Msg::Decrement)
                    } else {
                        Some(Msg::Increment)
                    };
                }
            }
        }
        None
    }

    fn title(&self) -> &str {
        "agpu — Counter"
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    AgpuApp::new(App { count: 0 })
        .with_options(ProgramOptions {
            width: 400.0,
            height: 200.0,
            ..Default::default()
        })
        .run()
}
