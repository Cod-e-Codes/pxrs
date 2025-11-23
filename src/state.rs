use crate::message::ExportFormat;
use iced::Color;
use iced::Rectangle;

#[derive(Debug, Clone)]
pub struct EditorState {
    pub canvas_width: u32,
    pub canvas_height: u32,
    pub current_tool: Tool,
    pub primary_color: Color,
    pub secondary_color: Color,
    pub brush_size: u32,
    pub zoom_level: f32,
    pub grid_visible: bool,
    pub layers: Vec<Layer>,
    pub active_layer_index: usize,
    pub history: History,
    pub selection: Option<Rectangle>,
    pub clipboard: Option<ClipboardData>,
    pub is_drawing: bool,
    pub last_pixel: Option<(u32, u32)>,
    pub selected_export_format: ExportFormat,
    pub is_selecting: bool,
    pub mirror_horizontal: bool,
    pub mirror_vertical: bool,
    pub used_colors: Vec<Color>,
}

impl Default for EditorState {
    fn default() -> Self {
        let width = 32;
        let height = 32;
        let layers = vec![Layer::new("Layer 1".to_string(), width, height)];

        Self {
            canvas_width: width,
            canvas_height: height,
            current_tool: Tool::Pencil,
            primary_color: Color::BLACK,
            secondary_color: Color::WHITE,
            brush_size: 1,
            zoom_level: 8.0,
            grid_visible: true,
            layers,
            active_layer_index: 0,
            history: History::new(),
            selection: None,
            clipboard: None,
            is_drawing: false,
            last_pixel: None,
            selected_export_format: ExportFormat::Png,
            is_selecting: false,
            mirror_horizontal: false,
            mirror_vertical: false,
            used_colors: vec![Color::BLACK, Color::WHITE],
        }
    }
}

impl EditorState {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            canvas_width: width,
            canvas_height: height,
            layers: vec![Layer::new("Layer 1".to_string(), width, height)],
            ..Default::default()
        }
    }

    pub fn active_layer_mut(&mut self) -> Option<&mut Layer> {
        self.layers.get_mut(self.active_layer_index)
    }

    pub fn active_layer(&self) -> Option<&Layer> {
        self.layers.get(self.active_layer_index)
    }

    pub fn get_pixel(&self, x: u32, y: u32) -> Color {
        if x >= self.canvas_width || y >= self.canvas_height {
            return Color::TRANSPARENT;
        }

        // Composite all visible layers from bottom to top
        let mut result = Color::TRANSPARENT;
        for layer in &self.layers {
            if !layer.visible {
                continue;
            }
            let pixel = layer.get_pixel(x, y);
            result = blend_color(result, pixel, layer.opacity);
        }
        result
    }

    pub fn set_pixel(&mut self, x: u32, y: u32, color: Color) {
        if let Some(layer) = self.active_layer_mut() {
            layer.set_pixel(x, y, color);
            self.add_used_color(color);
        }
    }

    pub fn add_layer(&mut self, name: String) {
        let layer = Layer::new(name, self.canvas_width, self.canvas_height);
        self.layers.push(layer);
        self.active_layer_index = self.layers.len() - 1;
    }

    pub fn delete_layer(&mut self, index: usize) {
        if self.layers.len() > 1 && index < self.layers.len() {
            self.layers.remove(index);
            if self.active_layer_index >= self.layers.len() {
                self.active_layer_index = self.layers.len().saturating_sub(1);
            }
        }
    }

    pub fn add_used_color(&mut self, color: Color) {
        // Don't add transparent colors
        if color.a < 0.01 {
            return;
        }

        // Check if color already exists (with tolerance for floating point)
        let exists = self.used_colors.iter().any(|c| {
            (c.r - color.r).abs() < 0.01
                && (c.g - color.g).abs() < 0.01
                && (c.b - color.b).abs() < 0.01
                && (c.a - color.a).abs() < 0.01
        });
        if !exists {
            self.used_colors.push(color);
            // Keep only the most recent 32 colors
            if self.used_colors.len() > 32 {
                self.used_colors.remove(0);
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tool {
    Pencil,
    Eraser,
    Fill,
    Selection,
    Eyedropper,
}

#[derive(Debug, Clone)]
pub struct Layer {
    pub name: String,
    pub pixels: Vec<u8>, // RGBA format
    pub width: u32,
    pub height: u32,
    pub visible: bool,
    pub opacity: f32,
}

impl Layer {
    pub fn new(name: String, width: u32, height: u32) -> Self {
        let size = (width * height * 4) as usize;
        let pixels = vec![0u8; size];
        // Initialize with transparent pixels (all zeros)
        Self {
            name,
            pixels,
            width,
            height,
            visible: true,
            opacity: 1.0,
        }
    }

    pub fn get_pixel(&self, x: u32, y: u32) -> Color {
        if x >= self.width || y >= self.height {
            return Color::TRANSPARENT;
        }
        let index = ((y * self.width + x) * 4) as usize;
        if index + 3 < self.pixels.len() {
            Color::from_rgba8(
                self.pixels[index],
                self.pixels[index + 1],
                self.pixels[index + 2],
                self.pixels[index + 3] as f32 / 255.0,
            )
        } else {
            Color::TRANSPARENT
        }
    }

    pub fn set_pixel(&mut self, x: u32, y: u32, color: Color) {
        if x >= self.width || y >= self.height {
            return;
        }
        let index = ((y * self.width + x) * 4) as usize;
        if index + 3 < self.pixels.len() {
            let rgba = color.into_rgba8();
            self.pixels[index] = rgba[0];
            self.pixels[index + 1] = rgba[1];
            self.pixels[index + 2] = rgba[2];
            self.pixels[index + 3] = rgba[3];
        }
    }

    pub fn get_pixel_buffer(&self) -> &[u8] {
        &self.pixels
    }
}

#[derive(Debug, Clone)]
pub struct History {
    pub commands: Vec<EditCommand>,
    pub current_index: usize,
}

impl History {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
            current_index: 0,
        }
    }

    pub fn push(&mut self, command: EditCommand) {
        // Remove any commands after current_index (when undoing and then doing new action)
        self.commands.truncate(self.current_index);
        self.commands.push(command);
        self.current_index += 1;
        // Limit history size
        if self.commands.len() > 100 {
            self.commands.remove(0);
            self.current_index -= 1;
        }
    }

    pub fn can_undo(&self) -> bool {
        self.current_index > 0
    }

    pub fn can_redo(&self) -> bool {
        self.current_index < self.commands.len()
    }

    pub fn undo(&mut self) -> Option<EditCommand> {
        if self.can_undo() {
            self.current_index -= 1;
            Some(self.commands[self.current_index].clone())
        } else {
            None
        }
    }

    pub fn redo(&mut self) -> Option<EditCommand> {
        if self.can_redo() {
            let command = self.commands[self.current_index].clone();
            self.current_index += 1;
            Some(command)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub enum EditCommand {
    PixelChange {
        layer_index: usize,
        x: u32,
        y: u32,
        old_color: Color,
        new_color: Color,
    },
    MultiPixelChange {
        layer_index: usize,
        changes: Vec<(u32, u32, Color, Color)>, // (x, y, old_color, new_color)
    },
}

fn blend_color(bottom: Color, top: Color, opacity: f32) -> Color {
    let bottom_rgba = bottom.into_rgba8();
    let top_rgba = top.into_rgba8();

    let br = bottom_rgba[0] as f32 / 255.0;
    let bg = bottom_rgba[1] as f32 / 255.0;
    let bb = bottom_rgba[2] as f32 / 255.0;
    let ba = bottom_rgba[3] as f32 / 255.0;

    let tr = top_rgba[0] as f32 / 255.0;
    let tg = top_rgba[1] as f32 / 255.0;
    let tb = top_rgba[2] as f32 / 255.0;
    let ta = top_rgba[3] as f32 / 255.0;

    let final_alpha = ta * opacity + ba * (1.0 - ta * opacity);
    if final_alpha == 0.0 {
        return Color::TRANSPARENT;
    }

    let r = (tr * ta * opacity + br * ba * (1.0 - ta * opacity)) / final_alpha;
    let g = (tg * ta * opacity + bg * ba * (1.0 - ta * opacity)) / final_alpha;
    let b = (tb * ta * opacity + bb * ba * (1.0 - ta * opacity)) / final_alpha;

    Color::from_rgba(r, g, b, final_alpha)
}

#[derive(Debug, Clone)]
pub struct ClipboardData {
    pub pixels: Vec<u8>,
    pub width: u32,
    pub height: u32,
}
