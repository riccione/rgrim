//! Shared egui styling helpers.

use eframe::egui::{Context, Theme};

/// Scales every named text style (body, button, heading, ...) by `factor`
/// for both themes. Each `run_native` session gets a fresh [`Context`], so
/// this must be called once per window at app startup.
pub fn apply_font_scale(ctx: &Context, factor: f32) {
    for theme in [Theme::Dark, Theme::Light] {
        ctx.style_mut_of(theme, |style| {
            for font in style.text_styles.values_mut() {
                font.size *= factor;
            }
        });
    }
}
