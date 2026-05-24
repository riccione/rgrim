use eframe::egui::Rect;

/// Represents a selected screen region captured by the sniper overlay.
#[derive(Clone, Debug)]
pub struct SelectionRect {
    pub region: Rect,
}
