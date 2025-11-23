use iced::Color;

pub fn color_to_rgba8(color: Color) -> [u8; 4] {
    color.into_rgba8()
}

pub fn rgba8_to_color(rgba: [u8; 4]) -> Color {
    Color::from_rgba8(rgba[0], rgba[1], rgba[2], rgba[3] as f32 / 255.0)
}

pub fn clamp_u32(value: i32, min: u32, max: u32) -> u32 {
    value.max(min as i32).min(max as i32) as u32
}

pub fn clamp_f32(value: f32, min: f32, max: f32) -> f32 {
    value.max(min).min(max)
}
