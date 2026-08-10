//! Egui/eframe backend for Dewey.
//!
//! Provides GPU-accelerated rendering via egui/wgpu through the [`Painter`]
//! trait. This module is only available with the "egui-backend" feature.

use crate::core::style::TextStyle;
use crate::core::{Color, Position, Rect, Size};
use crate::paint::{ImageData, Painter};

// ---------------------------------------------------------------------------
// EguiPainter — implements the Painter trait via egui's drawing API
// ---------------------------------------------------------------------------

/// A [`Painter`] implementation backed by egui's GPU-accelerated renderer.
pub struct EguiPainter {
    painter: egui::Painter,
    clip_stack: Vec<egui::Rect>,
}

impl EguiPainter {
    /// Create a new painter for the given egui context.
    pub fn new(ctx: &egui::Context) -> Self {
        let layer_id = egui::LayerId::new(egui::Order::Middle, egui::Id::new("dewey_painter"));
        let clip_rect = ctx.screen_rect();
        let painter = egui::Painter::new(ctx.clone(), layer_id, clip_rect);
        Self {
            painter,
            clip_stack: vec![clip_rect],
        }
    }
}

fn to_egui_rect(r: Rect) -> egui::Rect {
    egui::Rect::from_min_size(egui::pos2(r.x, r.y), egui::vec2(r.width, r.height))
}

fn to_egui_color(c: Color) -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(
        (c.r * 255.0) as u8,
        (c.g * 255.0) as u8,
        (c.b * 255.0) as u8,
        (c.a * 255.0) as u8,
    )
}

impl Painter for EguiPainter {
    fn fill_rect(&mut self, rect: Rect, color: Color, corner_radius: f32) {
        self.painter
            .rect_filled(to_egui_rect(rect), corner_radius, to_egui_color(color));
    }

    fn stroke_rect(&mut self, rect: Rect, color: Color, width: f32, corner_radius: f32) {
        self.painter.rect_stroke(
            to_egui_rect(rect),
            corner_radius,
            egui::Stroke::new(width, to_egui_color(color)),
            egui::StrokeKind::Inside,
        );
    }

    fn fill_circle(&mut self, center: Position, radius: f32, color: Color) {
        self.painter
            .circle_filled(egui::pos2(center.x, center.y), radius, to_egui_color(color));
    }

    fn stroke_circle(&mut self, center: Position, radius: f32, color: Color, width: f32) {
        self.painter.circle_stroke(
            egui::pos2(center.x, center.y),
            radius,
            egui::Stroke::new(width, to_egui_color(color)),
        );
    }

    fn line(&mut self, from: Position, to: Position, color: Color, width: f32) {
        self.painter.line_segment(
            [egui::pos2(from.x, from.y), egui::pos2(to.x, to.y)],
            egui::Stroke::new(width, to_egui_color(color)),
        );
    }

    fn text(&mut self, pos: Position, text: &str, style: &TextStyle) {
        let font_id = egui::FontId::proportional(style.font_size);
        self.painter.text(
            egui::pos2(pos.x, pos.y),
            egui::Align2::LEFT_TOP,
            text,
            font_id,
            to_egui_color(style.color),
        );
    }

    fn measure_text(&self, text: &str, style: &TextStyle) -> Size {
        let font_id = egui::FontId::proportional(style.font_size);
        let galley = self
            .painter
            .layout_no_wrap(text.to_string(), font_id, egui::Color32::WHITE);
        Size::new(galley.size().x, galley.size().y)
    }

    fn push_clip(&mut self, rect: Rect) {
        let egui_rect = to_egui_rect(rect);
        self.clip_stack.push(egui_rect);
        self.painter.set_clip_rect(egui_rect);
    }

    fn pop_clip(&mut self) {
        self.clip_stack.pop();
        if let Some(&clip) = self.clip_stack.last() {
            self.painter.set_clip_rect(clip);
        }
    }

    fn fill_path(&mut self, points: &[Position], color: Color) {
        // egui's convex-polygon primitive is the only filled-path shape it
        // offers. A concave path renders as its convex interpretation rather
        // than being dropped — wrong in detail but visible and in the right
        // place. A proper tessellator is the eventual fix and belongs here.
        if points.len() < 3 {
            return;
        }
        let shape = egui::epaint::PathShape::convex_polygon(
            points.iter().map(|p| egui::pos2(p.x, p.y)).collect(),
            to_egui_color(color),
            egui::Stroke::NONE,
        );
        self.painter.add(shape);
    }

    fn stroke_path(&mut self, points: &[Position], color: Color, width: f32) {
        if points.len() < 2 {
            return;
        }
        // One line shape rather than n segments, so egui mitres the joins
        // instead of leaving a notch at every corner.
        let shape = egui::epaint::PathShape::line(
            points.iter().map(|p| egui::pos2(p.x, p.y)).collect(),
            egui::Stroke::new(width, to_egui_color(color)),
        );
        self.painter.add(shape);
    }

    fn draw_image(&mut self, rect: Rect, image: &ImageData<'_>) {
        if image.width == 0 || image.height == 0 {
            return;
        }
        let size = [image.width as usize, image.height as usize];

        // egui wants exactly width * height * 4 bytes and panics inside
        // `from_rgba_unmultiplied` on a short slice, so pad rather than trust.
        let needed = size[0] * size[1] * 4;
        let pixels = if image.pixels.len() >= needed {
            std::borrow::Cow::Borrowed(&image.pixels[..needed])
        } else {
            let mut padded = image.pixels.to_vec();
            padded.resize(needed, 0);
            std::borrow::Cow::Owned(padded)
        };

        let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &pixels);
        // Keyed by content, so an image redrawn every frame uploads once
        // instead of allocating a fresh texture handle each time.
        let key = format!("dewey_img_{:016x}", content_key(&pixels, image.width));
        let handle =
            self.painter
                .ctx()
                .load_texture(key, color_image, egui::TextureOptions::LINEAR);
        self.painter.image(
            handle.id(),
            to_egui_rect(rect),
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            egui::Color32::WHITE,
        );
    }
}

/// A cheap content key for texture reuse: FNV-1a over a sample of the pixels.
///
/// Hashing every byte of a large scan on every frame would cost more than the
/// upload it saves, so this samples — enough to tell apart the images a
/// document actually contains, not a cryptographic digest.
fn content_key(pixels: &[u8], width: u32) -> u64 {
    const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01B3;

    let mut hash = OFFSET ^ u64::from(width);
    let step = (pixels.len() / 1024).max(1);
    for byte in pixels.iter().step_by(step) {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(PRIME);
    }
    hash ^ pixels.len() as u64
}

// ---------------------------------------------------------------------------
// EguiBackend — window configuration builder
// ---------------------------------------------------------------------------

/// Configuration for the egui backend.
pub struct EguiBackend {
    /// Native options forwarded to eframe.
    pub native_options: eframe::NativeOptions,
}

impl EguiBackend {
    /// Create a backend with default settings.
    pub fn new() -> Self {
        Self {
            native_options: eframe::NativeOptions {
                viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 600.0]),
                vsync: true,
                ..Default::default()
            },
        }
    }

    /// Set the initial window size.
    pub fn with_size(mut self, width: f32, height: f32) -> Self {
        self.native_options.viewport = self
            .native_options
            .viewport
            .with_inner_size([width, height]);
        self
    }

    /// Set the vsync mode.
    pub fn with_vsync(mut self, vsync: bool) -> Self {
        self.native_options.vsync = vsync;
        self
    }

    /// Set whether the window should start maximized.
    pub fn with_maximized(mut self, maximized: bool) -> Self {
        self.native_options.viewport = self.native_options.viewport.with_maximized(maximized);
        self
    }

    /// Set whether the window is resizable.
    pub fn with_resizable(mut self, resizable: bool) -> Self {
        self.native_options.viewport = self.native_options.viewport.with_resizable(resizable);
        self
    }

    /// Set the window title bar visibility.
    pub fn with_decorations(mut self, decorations: bool) -> Self {
        self.native_options.viewport = self.native_options.viewport.with_decorations(decorations);
        self
    }

    /// Set transparent window background.
    pub fn with_transparent(mut self, transparent: bool) -> Self {
        self.native_options.viewport = self.native_options.viewport.with_transparent(transparent);
        self
    }

    /// Set the minimum window size.
    pub fn with_min_size(mut self, width: f32, height: f32) -> Self {
        self.native_options.viewport = self
            .native_options
            .viewport
            .with_min_inner_size([width, height]);
        self
    }

    /// Set the window icon from RGBA pixel data.
    pub fn with_icon(mut self, rgba: Vec<u8>, width: u32, height: u32) -> Self {
        let icon = std::sync::Arc::new(egui::IconData {
            rgba,
            width,
            height,
        });
        self.native_options.viewport = self.native_options.viewport.with_icon(icon);
        self
    }

    /// Build the NativeOptions for use with eframe::run_native.
    pub fn build(self) -> eframe::NativeOptions {
        self.native_options
    }
}

impl Default for EguiBackend {
    fn default() -> Self {
        Self::new()
    }
}
