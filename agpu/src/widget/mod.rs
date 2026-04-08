//! Widget library for agpu — reusable UI components.
//!
//! Each widget implements the [`Widget`] trait and is fully discoverable
//! via the ontology system. Widgets render through the [`Painter`] trait
//! and register hitboxes for event routing.

mod button;
mod checkbox;
mod list;
mod menu;
mod modal;
mod progress;
mod select;
mod slider;
mod tabs;
mod text_area;
mod text_input;
mod tooltip;
mod tree;

pub use button::Button;
pub use checkbox::{Checkbox, Radio};
pub use list::{List, Table};
pub use menu::Menu;
pub use modal::Modal;
pub use progress::ProgressBar;
pub use select::Select;
pub use slider::Slider;
pub use tabs::{Panel, Tabs};
pub use text_area::TextArea;
pub use text_input::TextInput;
pub use tooltip::Tooltip;
pub use tree::TreeView;

use crate::core::Rect;
use crate::ontology::{Discoverable, UiNode};
use crate::paint::Painter;

/// Trait implemented by all widgets.
pub trait Widget: Discoverable {
    /// Render the widget into the given area using the painter.
    fn draw(&self, painter: &mut dyn Painter, area: Rect);

    /// Build a UI node for ontology registration.
    fn ui_node(&self) -> UiNode;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn button_draw_does_not_panic() {
        let btn = Button::new("click_me", "Click");
        let mut painter = crate::paint::NullPainter;
        btn.draw(&mut painter, Rect::new(0.0, 0.0, 100.0, 40.0));
    }

    #[test]
    fn text_input_draw_does_not_panic() {
        let input = TextInput::new("input_1");
        let mut painter = crate::paint::NullPainter;
        input.draw(&mut painter, Rect::new(0.0, 0.0, 200.0, 30.0));
    }

    #[test]
    fn checkbox_draw_does_not_panic() {
        let cb = Checkbox::new("cb_1", "Accept", false);
        let mut painter = crate::paint::NullPainter;
        cb.draw(&mut painter, Rect::new(0.0, 0.0, 200.0, 24.0));
    }

    #[test]
    fn radio_draw_does_not_panic() {
        let radio = Radio::new("r_1", "Option A", false);
        let mut painter = crate::paint::NullPainter;
        radio.draw(&mut painter, Rect::new(0.0, 0.0, 200.0, 24.0));
    }

    #[test]
    fn slider_draw_does_not_panic() {
        let slider = Slider::new("s_1", 0.5, 0.0, 1.0);
        let mut painter = crate::paint::NullPainter;
        slider.draw(&mut painter, Rect::new(0.0, 0.0, 200.0, 24.0));
    }

    #[test]
    fn select_draw_does_not_panic() {
        let select = Select::new("sel_1", vec!["A".into(), "B".into()]);
        let mut painter = crate::paint::NullPainter;
        select.draw(&mut painter, Rect::new(0.0, 0.0, 200.0, 30.0));
    }

    #[test]
    fn list_draw_does_not_panic() {
        let list = List::new("list_1", vec!["Item 1".into(), "Item 2".into()]);
        let mut painter = crate::paint::NullPainter;
        list.draw(&mut painter, Rect::new(0.0, 0.0, 200.0, 100.0));
    }

    #[test]
    fn table_draw_does_not_panic() {
        let table = Table::new(
            "table_1",
            vec!["Name".into(), "Age".into()],
            vec![vec!["Alice".into(), "30".into()]],
        );
        let mut painter = crate::paint::NullPainter;
        table.draw(&mut painter, Rect::new(0.0, 0.0, 400.0, 200.0));
    }

    #[test]
    fn tabs_draw_does_not_panic() {
        let tabs = Tabs::new("tabs_1", vec!["Tab A".into(), "Tab B".into()], 0);
        let mut painter = crate::paint::NullPainter;
        tabs.draw(&mut painter, Rect::new(0.0, 0.0, 400.0, 300.0));
    }

    #[test]
    fn modal_draw_does_not_panic() {
        let modal = Modal::new("modal_1", "Confirm?", true);
        let mut painter = crate::paint::NullPainter;
        modal.draw(&mut painter, Rect::new(0.0, 0.0, 800.0, 600.0));
    }

    #[test]
    fn tree_draw_does_not_panic() {
        let tree = TreeView::new(
            "tree_1",
            tree::TreeNode::new("root", "Root"),
        );
        let mut painter = crate::paint::NullPainter;
        tree.draw(&mut painter, Rect::new(0.0, 0.0, 200.0, 200.0));
    }

    #[test]
    fn progress_draw_does_not_panic() {
        let bar = ProgressBar::new("prog_1", 0.75);
        let mut painter = crate::paint::NullPainter;
        bar.draw(&mut painter, Rect::new(0.0, 0.0, 200.0, 20.0));
    }

    #[test]
    fn menu_draw_does_not_panic() {
        let menu = Menu::new("menu_1", vec![menu::MenuItem::item("open", "Open")]);
        let mut painter = crate::paint::NullPainter;
        menu.draw(&mut painter, Rect::new(0.0, 0.0, 200.0, 300.0));
    }

    #[test]
    fn text_area_draw_does_not_panic() {
        let ta = TextArea::new("ta_1");
        let mut painter = crate::paint::NullPainter;
        ta.draw(&mut painter, Rect::new(0.0, 0.0, 300.0, 200.0));
    }

    #[test]
    fn tooltip_draw_does_not_panic() {
        let tt = Tooltip::new("tt_1", "Help text");
        let mut painter = crate::paint::NullPainter;
        tt.draw(&mut painter, Rect::new(100.0, 100.0, 150.0, 30.0));
    }
}
