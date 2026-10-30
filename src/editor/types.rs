use eframe::egui::{self, Pos2};

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum DrawTool {
    Pen,
    Highlighter,
}

impl DrawTool {
    pub fn thickness(self) -> f32 {
        match self {
            DrawTool::Pen => 3.0,
            DrawTool::Highlighter => 24.0,
        }
    }

    /// Opacity is a property of the tool, not of the picked color:
    /// the highlighter is always translucent.
    fn alpha(self) -> u8 {
        match self {
            DrawTool::Pen => 255,
            DrawTool::Highlighter => 80,
        }
    }

    pub fn default_color(self) -> egui::Color32 {
        match self {
            DrawTool::Pen => egui::Color32::RED,
            DrawTool::Highlighter => egui::Color32::from_rgb(255, 255, 0),
        }
    }

    /// Combines a swatch color with the tool's fixed opacity. The pick is
    /// unmultiplied first, so translucent inputs still yield the intended
    /// hue; opaque picks pass through byte-identical.
    pub fn stroke_color(self, picked: egui::Color32) -> egui::Color32 {
        let [r, g, b, _] = picked.to_srgba_unmultiplied();
        egui::Color32::from_rgba_unmultiplied(r, g, b, self.alpha())
    }
}

/// User-selectable colors for each drawing tool. Survives across
/// capture sessions so the next "New" screenshot keeps the chosen hues.
///
/// Colors are kept opaque: the editor's color picker is the only writer,
/// and tools apply their own opacity when creating strokes.
#[derive(Clone, Copy, PartialEq)]
pub struct ToolPalette {
    pen: egui::Color32,
    highlighter: egui::Color32,
}

impl Default for ToolPalette {
    fn default() -> Self {
        Self {
            pen: DrawTool::Pen.default_color(),
            highlighter: DrawTool::Highlighter.default_color(),
        }
    }
}

impl ToolPalette {
    pub(crate) fn color(self, tool: DrawTool) -> egui::Color32 {
        match tool {
            DrawTool::Pen => self.pen,
            DrawTool::Highlighter => self.highlighter,
        }
    }

    pub(crate) fn color_mut(&mut self, tool: DrawTool) -> &mut egui::Color32 {
        match tool {
            DrawTool::Pen => &mut self.pen,
            DrawTool::Highlighter => &mut self.highlighter,
        }
    }
}

pub(crate) struct Stroke {
    pub points: Vec<Pos2>,
    pub color: egui::Color32,
    pub thickness: f32,
}

#[cfg(test)]
mod tests {
    use super::{DrawTool, ToolPalette};
    use eframe::egui::Color32;

    #[test]
    fn highlighter_applies_fixed_translucency_to_picked_colors() {
        let picked = Color32::from_rgb(255, 0, 255);

        assert_eq!(
            DrawTool::Highlighter.stroke_color(picked),
            Color32::from_rgba_unmultiplied(255, 0, 255, 80)
        );
    }

    #[test]
    fn pen_keeps_picked_colors_opaque() {
        let picked = Color32::from_rgb(0, 128, 255);

        assert_eq!(DrawTool::Pen.stroke_color(picked), picked);
    }

    #[test]
    fn translucent_picks_are_unmultiplied_before_tool_opacity() {
        let picked = Color32::from_rgba_unmultiplied(255, 0, 0, 128);

        assert_eq!(DrawTool::Pen.stroke_color(picked), Color32::RED);
        assert_eq!(
            DrawTool::Highlighter.stroke_color(picked),
            Color32::from_rgba_unmultiplied(255, 0, 0, 80)
        );
    }

    #[test]
    fn palette_defaults_reproduce_the_legacy_tool_colors() {
        let palette = ToolPalette::default();

        assert_eq!(
            DrawTool::Pen.stroke_color(palette.color(DrawTool::Pen)),
            Color32::RED
        );
        assert_eq!(
            DrawTool::Highlighter.stroke_color(palette.color(DrawTool::Highlighter)),
            Color32::from_rgba_unmultiplied(255, 255, 0, 80)
        );
    }
}
