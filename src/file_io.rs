use crate::message::ExportFormat;
use crate::state::EditorState;
use std::path::Path;

pub fn save_image(state: &EditorState, path: &Path, format: ExportFormat) -> Result<(), String> {
    // Composite all visible layers into a single image
    let width = state.canvas_width;
    let height = state.canvas_height;
    let mut rgba_data = vec![0u8; (width * height * 4) as usize];

    // Start with transparent background
    for pixel in rgba_data.chunks_exact_mut(4) {
        pixel[0] = 0;
        pixel[1] = 0;
        pixel[2] = 0;
        pixel[3] = 0;
    }

    // Composite layers from bottom to top
    for layer in &state.layers {
        if !layer.visible {
            continue;
        }

        // Use get_pixel_buffer for efficient pixel access
        let layer_pixels = layer.get_pixel_buffer();
        for y in 0..height {
            for x in 0..width {
                let index = ((y * width + x) * 4) as usize;
                if index + 3 >= layer_pixels.len() {
                    continue;
                }

                let r = layer_pixels[index];
                let g = layer_pixels[index + 1];
                let b = layer_pixels[index + 2];
                let a = layer_pixels[index + 3];

                let out_index = ((y * width + x) * 4) as usize;
                if out_index + 3 < rgba_data.len() {
                    // Alpha blend
                    let alpha = (a as f32 / 255.0) * layer.opacity;
                    let inv_alpha = 1.0 - alpha;

                    rgba_data[out_index] =
                        (r as f32 * alpha + rgba_data[out_index] as f32 * inv_alpha) as u8;
                    rgba_data[out_index + 1] =
                        (g as f32 * alpha + rgba_data[out_index + 1] as f32 * inv_alpha) as u8;
                    rgba_data[out_index + 2] =
                        (b as f32 * alpha + rgba_data[out_index + 2] as f32 * inv_alpha) as u8;
                    rgba_data[out_index + 3] = (rgba_data[out_index + 3] as f32
                        + a as f32 * layer.opacity)
                        .min(255.0) as u8;
                }
            }
        }
    }

    // Convert to image crate format
    let img = image::RgbaImage::from_raw(width, height, rgba_data)
        .ok_or("Failed to create image from pixel data")?;

    match format {
        ExportFormat::Png => {
            img.save(path)
                .map_err(|e| format!("Failed to save PNG: {}", e))?;
        }
        ExportFormat::Gif => {
            // GIF doesn't support RGBA directly, convert to RGB
            let rgb_img = image::DynamicImage::ImageRgba8(img).to_rgb8();
            rgb_img
                .save(path)
                .map_err(|e| format!("Failed to save GIF: {}", e))?;
        }
        ExportFormat::Bmp => {
            img.save(path)
                .map_err(|e| format!("Failed to save BMP: {}", e))?;
        }
    }

    Ok(())
}

pub fn load_image(path: &Path) -> Result<(u32, u32, Vec<u8>), String> {
    let img = image::open(path).map_err(|e| format!("Failed to open image: {}", e))?;

    let rgba_img = img.to_rgba8();
    let (width, height) = rgba_img.dimensions();
    // Use get_pixel_buffer equivalent - just get raw pixels
    let pixels = rgba_img.into_raw();

    Ok((width, height, pixels))
}
