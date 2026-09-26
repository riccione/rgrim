use eframe::egui::{self, Pos2};

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum DrawTool {
    Pen,
    Highlighter,
}

impl DrawTool {
    pub fn drawing_properties(self) -> (egui::Color32, f32) {
        match self {
            DrawTool::Pen => (egui::Color32::RED, 3.0),
            DrawTool::Highlighter => (
                egui::Color32::from_rgba_premultiplied(255, 255, 0, 80),
                24.0,
            ),
        }
    }
}

pub(crate) struct Stroke {
    pub points: Vec<Pos2>,
    pub color: egui::Color32,
    pub thickness: f32,
}
