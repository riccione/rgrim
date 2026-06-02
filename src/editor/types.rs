use eframe::egui::{self, Pos2};

#[derive(PartialEq)]
pub(crate) enum Tool {
    None,
    Pen,
    Highlighter,
}

impl Tool {
    pub fn drawing_properties(&self) -> (egui::Color32, f32) {
        match self {
            Tool::Pen => (egui::Color32::RED, 3.0),
            Tool::Highlighter => (
                egui::Color32::from_rgba_premultiplied(255, 255, 0, 80),
                24.0,
            ),
            Tool::None => unreachable!("drawing_properties called on Tool::None"),
        }
    }
}

pub(crate) struct Stroke {
    pub points: Vec<Pos2>,
    pub color: egui::Color32,
    pub thickness: f32,
}
