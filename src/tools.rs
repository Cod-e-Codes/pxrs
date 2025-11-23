use crate::state::EditorState;
use crate::utils;
use iced::{Color, Rectangle};

fn get_brush_pixels(
    x: u32,
    y: u32,
    size: u32,
    canvas_width: u32,
    canvas_height: u32,
) -> Vec<(u32, u32)> {
    let mut pixels = Vec::new();
    let radius = (size / 2) as i32;

    for dy in -(radius)..=(radius) {
        for dx in -(radius)..=(radius) {
            let px = x as i32 + dx;
            let py = y as i32 + dy;

            if px >= 0 && py >= 0 && px < canvas_width as i32 && py < canvas_height as i32 {
                pixels.push((px as u32, py as u32));
            }
        }
    }

    pixels
}

fn get_mirrored_positions(state: &EditorState, x: u32, y: u32) -> Vec<(u32, u32)> {
    let mut positions = vec![(x, y)];

    if state.mirror_horizontal {
        let mirrored_x = state.canvas_width.saturating_sub(1).saturating_sub(x);
        positions.push((mirrored_x, y));
    }

    if state.mirror_vertical {
        let mirrored_y = state.canvas_height.saturating_sub(1).saturating_sub(y);
        positions.push((x, mirrored_y));
    }

    if state.mirror_horizontal && state.mirror_vertical {
        let mirrored_x = state.canvas_width.saturating_sub(1).saturating_sub(x);
        let mirrored_y = state.canvas_height.saturating_sub(1).saturating_sub(y);
        positions.push((mirrored_x, mirrored_y));
    }

    // Remove duplicates
    positions.sort();
    positions.dedup();
    positions
}

pub fn apply_pencil(state: &mut EditorState, x: u32, y: u32) {
    if x >= state.canvas_width || y >= state.canvas_height {
        return;
    }

    let primary_color = state.primary_color;
    let layer_index = state.active_layer_index;
    let brush_size = state.brush_size;

    let mut all_positions = Vec::new();

    // Get brush pixels
    let brush_pixels = get_brush_pixels(x, y, brush_size, state.canvas_width, state.canvas_height);

    // Apply mirroring to each brush pixel
    for (bx, by) in brush_pixels {
        let mirrored = get_mirrored_positions(state, bx, by);
        all_positions.extend(mirrored);
    }

    // Remove duplicates
    all_positions.sort();
    all_positions.dedup();

    // Collect all changes for undo
    let mut changes = Vec::new();

    for (px, py) in all_positions {
        if px >= state.canvas_width || py >= state.canvas_height {
            continue;
        }

        let old_color = if let Some(layer) = state.active_layer() {
            layer.get_pixel(px, py)
        } else {
            continue;
        };

        // Use EditorState::set_pixel for consistency
        state.set_pixel(px, py, primary_color);

        changes.push((px, py, old_color, primary_color));
    }

    // Record changes for undo
    if changes.len() == 1 {
        let (px, py, old_color, new_color) = changes[0];
        state.history.push(crate::state::EditCommand::PixelChange {
            layer_index,
            x: px,
            y: py,
            old_color,
            new_color,
        });
    } else if !changes.is_empty() {
        state
            .history
            .push(crate::state::EditCommand::MultiPixelChange {
                layer_index,
                changes: changes.into_iter().collect(),
            });
    }
}

pub fn apply_eraser(state: &mut EditorState, x: u32, y: u32) {
    if x >= state.canvas_width || y >= state.canvas_height {
        return;
    }

    let layer_index = state.active_layer_index;
    let brush_size = state.brush_size;
    let new_color = Color::TRANSPARENT;

    let mut all_positions = Vec::new();

    // Get brush pixels
    let brush_pixels = get_brush_pixels(x, y, brush_size, state.canvas_width, state.canvas_height);

    // Apply mirroring to each brush pixel
    for (bx, by) in brush_pixels {
        let mirrored = get_mirrored_positions(state, bx, by);
        all_positions.extend(mirrored);
    }

    // Remove duplicates
    all_positions.sort();
    all_positions.dedup();

    // Collect all changes for undo
    let mut changes = Vec::new();

    for (px, py) in all_positions {
        if px >= state.canvas_width || py >= state.canvas_height {
            continue;
        }

        let old_color = if let Some(layer) = state.active_layer() {
            layer.get_pixel(px, py)
        } else {
            continue;
        };

        // Use EditorState::set_pixel for consistency
        state.set_pixel(px, py, new_color);

        changes.push((px, py, old_color, new_color));
    }

    // Record changes for undo
    if changes.len() == 1 {
        let (px, py, old_color, new_color) = changes[0];
        state.history.push(crate::state::EditCommand::PixelChange {
            layer_index,
            x: px,
            y: py,
            old_color,
            new_color,
        });
    } else if !changes.is_empty() {
        state
            .history
            .push(crate::state::EditCommand::MultiPixelChange {
                layer_index,
                changes: changes.into_iter().collect(),
            });
    }
}

pub fn apply_eyedropper(state: &mut EditorState, x: u32, y: u32) {
    if x >= state.canvas_width || y >= state.canvas_height {
        return;
    }

    // Get the composited color at this position
    let color = state.get_pixel(x, y);

    // Only pick non-transparent colors
    if color.a > 0.01 {
        state.primary_color = color;
        state.add_used_color(color);
    }
}

pub fn apply_fill(state: &mut EditorState, x: u32, y: u32) {
    if x >= state.canvas_width || y >= state.canvas_height {
        return;
    }

    let primary_color = state.primary_color;
    let canvas_width = state.canvas_width;
    let canvas_height = state.canvas_height;
    let layer_index = state.active_layer_index;

    if let Some(layer) = state.active_layer_mut() {
        let target_color = layer.get_pixel(x, y);

        // Don't fill if target is already the fill color
        if target_color == primary_color {
            return;
        }

        // Flood fill using BFS
        let mut changes = Vec::new();
        let mut queue = std::collections::VecDeque::new();
        let mut visited = std::collections::HashSet::new();

        queue.push_back((x, y));
        visited.insert((x, y));

        while let Some((cx, cy)) = queue.pop_front() {
            if cx >= canvas_width || cy >= canvas_height {
                continue;
            }

            let current_color = layer.get_pixel(cx, cy);
            if current_color != target_color {
                continue;
            }

            let old_color = current_color;
            changes.push((cx, cy, old_color, primary_color));
            layer.set_pixel(cx, cy, primary_color);

            // Add neighbors
            for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                let nx = cx as i32 + dx;
                let ny = cy as i32 + dy;
                if nx >= 0 && ny >= 0 {
                    let nx = nx as u32;
                    let ny = ny as u32;
                    if !visited.contains(&(nx, ny)) && nx < canvas_width && ny < canvas_height {
                        visited.insert((nx, ny));
                        queue.push_back((nx, ny));
                    }
                }
            }
        }

        if !changes.is_empty() {
            state
                .history
                .push(crate::state::EditCommand::MultiPixelChange {
                    layer_index,
                    changes,
                });
        }
    }
}

pub fn get_selection_pixels(state: &EditorState, selection: Rectangle) -> Option<Vec<u8>> {
    let start_x = utils::clamp_u32(selection.x as i32, 0, state.canvas_width);
    let start_y = utils::clamp_u32(selection.y as i32, 0, state.canvas_height);
    let end_x = utils::clamp_u32(
        (selection.x + selection.width) as i32,
        0,
        state.canvas_width,
    );
    let end_y = utils::clamp_u32(
        (selection.y + selection.height) as i32,
        0,
        state.canvas_height,
    );

    if start_x >= end_x || start_y >= end_y {
        return None;
    }

    let width = end_x - start_x;
    let height = end_y - start_y;
    let mut pixels = vec![0u8; (width * height * 4) as usize];

    for y in start_y..end_y {
        for x in start_x..end_x {
            let color = state.get_pixel(x, y);
            let rgba = utils::color_to_rgba8(color);
            let index = (((y - start_y) * width + (x - start_x)) * 4) as usize;
            if index + 3 < pixels.len() {
                pixels[index] = rgba[0];
                pixels[index + 1] = rgba[1];
                pixels[index + 2] = rgba[2];
                pixels[index + 3] = rgba[3];
            }
        }
    }

    Some(pixels)
}

pub fn paste_pixels(
    state: &mut EditorState,
    pixels: &[u8],
    start_x: u32,
    start_y: u32,
    width: u32,
    height: u32,
) {
    let canvas_width = state.canvas_width;
    let canvas_height = state.canvas_height;
    let layer_index = state.active_layer_index;

    if let Some(layer) = state.active_layer_mut() {
        let mut changes = Vec::new();

        for y in 0..height {
            for x in 0..width {
                let canvas_x = start_x + x;
                let canvas_y = start_y + y;

                if canvas_x >= canvas_width || canvas_y >= canvas_height {
                    continue;
                }

                let index = ((y * width + x) * 4) as usize;
                if index + 3 < pixels.len() {
                    let old_color = layer.get_pixel(canvas_x, canvas_y);
                    let rgba = [
                        pixels[index],
                        pixels[index + 1],
                        pixels[index + 2],
                        pixels[index + 3],
                    ];
                    let new_color = utils::rgba8_to_color(rgba);

                    changes.push((canvas_x, canvas_y, old_color, new_color));
                    layer.set_pixel(canvas_x, canvas_y, new_color);
                }
            }
        }

        if !changes.is_empty() {
            state
                .history
                .push(crate::state::EditCommand::MultiPixelChange {
                    layer_index,
                    changes,
                });
        }
    }
}
