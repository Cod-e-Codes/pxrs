mod canvas;
mod file_io;
mod message;
mod state;
mod tools;
mod ui;
mod utils;

use iced::Task;
use message::Message;
use state::EditorState;

fn main() -> iced::Result {
    iced::application("Pixel Art Editor", update, view)
        .subscription(subscription)
        .run()
}

fn subscription(_state: &EditorState) -> iced::Subscription<Message> {
    use iced::keyboard;
    use iced::keyboard::key;

    keyboard::on_key_press(|key, modifiers| {
        match (key.as_ref(), modifiers) {
            (key::Key::Character(c), keyboard::Modifiers::CTRL) if c.eq_ignore_ascii_case("z") => {
                if modifiers.contains(keyboard::Modifiers::SHIFT) {
                    Some(Message::Redo)
                } else {
                    Some(Message::Undo)
                }
            }
            (key::Key::Character(c), keyboard::Modifiers::CTRL) if c.eq_ignore_ascii_case("y") => {
                Some(Message::Redo)
            }
            (key::Key::Character(c), keyboard::Modifiers::CTRL) if c.eq_ignore_ascii_case("c") => {
                Some(Message::CopySelection)
            }
            (key::Key::Character(c), keyboard::Modifiers::CTRL) if c.eq_ignore_ascii_case("v") => {
                // Paste at current mouse position - for now paste at center
                Some(Message::PasteSelection { x: 16, y: 16 })
            }
            (key::Key::Character(c), keyboard::Modifiers::CTRL) if c.eq_ignore_ascii_case("x") => {
                Some(Message::CutSelection)
            }
            (key::Key::Character(c), keyboard::Modifiers::CTRL) if c.eq_ignore_ascii_case("a") => {
                // Select all - create selection covering entire canvas
                Some(Message::SelectionStarted { x: 0.0, y: 0.0 })
            }
            (key::Key::Named(key::Named::Delete), _)
            | (key::Key::Named(key::Named::Backspace), _) => {
                // Clear selection or delete key
                Some(Message::SelectionCleared)
            }
            _ => None,
        }
    })
}

fn update(state: &mut EditorState, message: Message) -> Task<Message> {
    match message {
        Message::ToolSelected(tool) => {
            state.current_tool = tool;
        }
        Message::PrimaryColorChanged(color) => {
            state.primary_color = color;
        }
        Message::SecondaryColorChanged(color) => {
            state.secondary_color = color;
        }
        Message::ColorPicked(color) => {
            // Color picker clicked - swap primary and secondary or set primary
            state.primary_color = color;
        }
        Message::UsedColorPicked(color) => {
            state.primary_color = color;
        }
        Message::BrushSizeChanged(size) => {
            state.brush_size = size.clamp(1, 20);
        }
        Message::CanvasResized { width, height } => {
            state.canvas_width = width;
            state.canvas_height = height;
            // Resize all layers
            for layer in &mut state.layers {
                let new_pixels = vec![0u8; (width * height * 4) as usize];
                layer.pixels = new_pixels;
                layer.width = width;
                layer.height = height;
            }
        }
        Message::CanvasCleared => {
            for layer in &mut state.layers {
                layer.pixels.fill(0);
            }
        }
        Message::LayerAdded(name) => {
            state.add_layer(name);
        }
        Message::LayerDeleted(index) => {
            state.delete_layer(index);
        }
        Message::LayerMoved { from, to } => {
            if from < state.layers.len() && to < state.layers.len() {
                let layer = state.layers.remove(from);
                state.layers.insert(to, layer);
                if state.active_layer_index == from {
                    state.active_layer_index = to;
                } else if state.active_layer_index == to && from < to {
                    state.active_layer_index -= 1;
                } else if state.active_layer_index == to && from > to {
                    state.active_layer_index += 1;
                }
            }
        }
        Message::LayerVisibilityToggled(index) => {
            if let Some(layer) = state.layers.get_mut(index) {
                layer.visible = !layer.visible;
            }
        }
        Message::LayerSelected(index) => {
            if index < state.layers.len() {
                state.active_layer_index = index;
            }
        }
        Message::LayerOpacityChanged { index, opacity } => {
            if let Some(layer) = state.layers.get_mut(index) {
                layer.opacity = utils::clamp_f32(opacity, 0.0, 1.0);
            }
        }
        Message::LayerRenamed { index, name } => {
            if let Some(layer) = state.layers.get_mut(index)
                && !name.is_empty()
            {
                layer.name = name;
            }
        }
        Message::DrawingStarted { x, y } => {
            let is_selection_tool = matches!(state.current_tool, state::Tool::Selection);
            let is_eyedropper = matches!(state.current_tool, state::Tool::Eyedropper);

            state.is_drawing = !is_selection_tool && !is_eyedropper;
            state.is_selecting = is_selection_tool;
            state.last_pixel = Some((x, y));

            match state.current_tool {
                state::Tool::Pencil => {
                    tools::apply_pencil(state, x, y);
                }
                state::Tool::Eraser => {
                    tools::apply_eraser(state, x, y);
                }
                state::Tool::Fill => {
                    tools::apply_fill(state, x, y);
                }
                state::Tool::Selection => {
                    state.selection = Some(iced::Rectangle {
                        x: x as f32,
                        y: y as f32,
                        width: 0.0,
                        height: 0.0,
                    });
                }
                state::Tool::Eyedropper => {
                    tools::apply_eyedropper(state, x, y);
                }
            }
        }
        Message::PixelDrawn { x, y } => {
            if state.is_drawing {
                // Prevent drawing the same pixel twice in a row
                if state.last_pixel != Some((x, y)) {
                    state.last_pixel = Some((x, y));
                    match state.current_tool {
                        state::Tool::Pencil => {
                            tools::apply_pencil(state, x, y);
                        }
                        state::Tool::Eraser => {
                            tools::apply_eraser(state, x, y);
                        }
                        state::Tool::Fill | state::Tool::Selection | state::Tool::Eyedropper => {
                            // Fill only happens on click, not drag
                            // Selection is handled by SelectionUpdated messages
                            // Eyedropper only works on click
                        }
                    }
                }
            }
        }
        Message::DrawingEnded => {
            state.is_drawing = false;
            state.last_pixel = None;
            if !matches!(state.current_tool, state::Tool::Selection) {
                state.is_selecting = false;
            }
        }
        Message::FileNew => {
            *state = EditorState::new(32, 32);
        }
        Message::FileOpen => {
            return Task::perform(
                async {
                    let file = rfd::AsyncFileDialog::new()
                        .add_filter("Image files", &["png", "jpg", "jpeg", "gif", "bmp"])
                        .add_filter("PNG", &["png"])
                        .add_filter("JPEG", &["jpg", "jpeg"])
                        .add_filter("GIF", &["gif"])
                        .add_filter("BMP", &["bmp"])
                        .pick_file()
                        .await;

                    if let Some(file) = file {
                        let path = file.path().to_string_lossy().to_string();
                        let path_clone = path.clone();
                        match file_io::load_image(file.path()) {
                            Ok((_width, _height, pixels)) => Message::FileLoaded {
                                path: path_clone,
                                data: pixels,
                            },
                            Err(e) => {
                                eprintln!("Failed to load image: {}", e);
                                Message::None
                            }
                        }
                    } else {
                        Message::None
                    }
                },
                |msg| msg,
            );
        }
        Message::FileSave => {
            let format = state.selected_export_format;
            let extension = match format {
                message::ExportFormat::Png => "png",
                message::ExportFormat::Gif => "gif",
                message::ExportFormat::Bmp => "bmp",
            };

            return Task::perform(
                async move {
                    let file = rfd::AsyncFileDialog::new()
                        .add_filter(format!("{} files", extension.to_uppercase()), &[extension])
                        .add_filter("All files", &["*"])
                        .set_file_name(format!("output.{}", extension))
                        .save_file()
                        .await;

                    if let Some(file) = file {
                        let path = file.path().to_string_lossy().to_string();
                        Message::FileSaveDialogResult { path, format }
                    } else {
                        Message::None
                    }
                },
                |msg| msg,
            );
        }
        Message::FileSaveDialogResult { path, format } => {
            use std::path::Path;
            if let Err(e) = file_io::save_image(state, Path::new(&path), format) {
                eprintln!("Failed to save: {}", e);
            } else {
                // Emit FileSaved message
                return Task::perform(
                    async move { Message::FileSaved { path: path.clone() } },
                    |msg| msg,
                );
            }
        }
        Message::ExportFormatSelected(format) => {
            state.selected_export_format = format;
        }
        Message::FileLoaded { path, data } => {
            // Use the data directly if provided, otherwise load from path
            let (width, height, pixels) = if !data.is_empty() {
                // Calculate dimensions from data (assuming square for now)
                let pixel_count = data.len() / 4;
                let size = (pixel_count as f32).sqrt() as u32;
                (size, size, data)
            } else {
                use std::path::Path;
                match file_io::load_image(Path::new(&path)) {
                    Ok(result) => result,
                    Err(e) => {
                        eprintln!("Failed to load image: {}", e);
                        return Task::none();
                    }
                }
            };
            // Create a new layer with the loaded image
            let mut new_layer = state::Layer::new("Imported".to_string(), width, height);
            new_layer.pixels = pixels;
            state.layers.push(new_layer);
            state.active_layer_index = state.layers.len() - 1;
            // Resize canvas if needed
            if width > state.canvas_width || height > state.canvas_height {
                state.canvas_width = width.max(state.canvas_width);
                state.canvas_height = height.max(state.canvas_height);
                // Resize all existing layers to match new canvas size
                for layer in &mut state.layers {
                    if layer.width != state.canvas_width || layer.height != state.canvas_height {
                        let new_pixels =
                            vec![0u8; (state.canvas_width * state.canvas_height * 4) as usize];
                        layer.pixels = new_pixels;
                        layer.width = state.canvas_width;
                        layer.height = state.canvas_height;
                    }
                }
            }
        }
        Message::FileSaved { path } => {
            // File saved successfully - log the path
            eprintln!("File saved successfully: {}", path);
        }
        Message::Undo => {
            if let Some(command) = state.history.undo() {
                apply_undo_command(state, command);
            }
        }
        Message::Redo => {
            if let Some(command) = state.history.redo() {
                apply_redo_command(state, command);
            }
        }
        Message::ZoomChanged(zoom) => {
            state.zoom_level = utils::clamp_f32(zoom, 1.0, 32.0);
        }
        Message::ZoomIn => {
            state.zoom_level = (state.zoom_level + 1.0).min(32.0);
        }
        Message::ZoomOut => {
            state.zoom_level = (state.zoom_level - 1.0).max(1.0);
        }
        Message::GridToggled => {
            state.grid_visible = !state.grid_visible;
        }
        Message::PanChanged { x, y } => {
            // Store pan offset for future use
            // Pan can be used for canvas offset when implementing panning
            // For now, pan is handled by canvas scrolling, but we store the values
            let _pan_x = x;
            let _pan_y = y;
        }
        Message::SelectionStarted { x, y } => {
            state.is_selecting = true;
            state.selection = Some(iced::Rectangle {
                x,
                y,
                width: 0.0,
                height: 0.0,
            });
        }
        Message::SelectionUpdated { x, y } => {
            if state.is_selecting {
                if let Some(sel) = &mut state.selection {
                    sel.width = x - sel.x;
                    sel.height = y - sel.y;
                } else if state.current_tool == state::Tool::Selection {
                    // Start selection if not already started
                    state.selection = Some(iced::Rectangle {
                        x,
                        y,
                        width: 0.0,
                        height: 0.0,
                    });
                }
            }
        }
        Message::SelectionEnded => {
            state.is_selecting = false;
            if let Some(sel) = &mut state.selection {
                if sel.width < 0.0 {
                    sel.x += sel.width;
                    sel.width = sel.width.abs();
                }
                if sel.height < 0.0 {
                    sel.y += sel.height;
                    sel.height = sel.height.abs();
                }
            }
        }
        Message::SelectionCleared => {
            state.selection = None;
            state.is_selecting = false;
        }
        Message::CopySelection => {
            if let Some(selection) = state.selection
                && let Some(pixels) = tools::get_selection_pixels(state, selection)
            {
                // Calculate dimensions the same way as get_selection_pixels does
                let start_x = crate::utils::clamp_u32(selection.x as i32, 0, state.canvas_width);
                let start_y = crate::utils::clamp_u32(selection.y as i32, 0, state.canvas_height);
                let end_x = crate::utils::clamp_u32(
                    (selection.x + selection.width) as i32,
                    0,
                    state.canvas_width,
                );
                let end_y = crate::utils::clamp_u32(
                    (selection.y + selection.height) as i32,
                    0,
                    state.canvas_height,
                );
                let width = end_x.saturating_sub(start_x);
                let height = end_y.saturating_sub(start_y);
                state.clipboard = Some(state::ClipboardData {
                    pixels,
                    width,
                    height,
                });
            }
        }
        Message::PasteSelection { x, y } => {
            if let Some(clipboard) = state.clipboard.clone() {
                tools::paste_pixels(
                    state,
                    &clipboard.pixels,
                    x,
                    y,
                    clipboard.width,
                    clipboard.height,
                );
            }
        }
        Message::CutSelection => {
            if let Some(selection) = state.selection {
                let canvas_width = state.canvas_width;
                let canvas_height = state.canvas_height;
                if let Some(pixels) = tools::get_selection_pixels(state, selection) {
                    // Calculate dimensions the same way as get_selection_pixels does
                    let start_x = crate::utils::clamp_u32(selection.x as i32, 0, canvas_width);
                    let start_y = crate::utils::clamp_u32(selection.y as i32, 0, canvas_height);
                    let end_x = crate::utils::clamp_u32(
                        (selection.x + selection.width) as i32,
                        0,
                        canvas_width,
                    );
                    let end_y = crate::utils::clamp_u32(
                        (selection.y + selection.height) as i32,
                        0,
                        canvas_height,
                    );
                    let width = end_x.saturating_sub(start_x);
                    let height = end_y.saturating_sub(start_y);
                    state.clipboard = Some(state::ClipboardData {
                        pixels,
                        width,
                        height,
                    });
                    // Clear the selected area
                    if let Some(layer) = state.active_layer_mut() {
                        for y in start_y..end_y {
                            for x in start_x..end_x {
                                layer.set_pixel(x, y, iced::Color::TRANSPARENT);
                            }
                        }
                    }
                }
            }
        }
        Message::CanvasEvent(event) => {
            // Forward canvas events if needed
            // Most are handled directly by canvas program
            // Handle any additional canvas events here if needed
            let _ = event;
        }
        Message::MirrorHorizontalToggled => {
            state.mirror_horizontal = !state.mirror_horizontal;
        }
        Message::MirrorVerticalToggled => {
            state.mirror_vertical = !state.mirror_vertical;
        }
        Message::None => {
            // No-op message
        }
    }

    Task::none()
}

fn apply_undo_command(state: &mut EditorState, command: state::EditCommand) {
    match command {
        state::EditCommand::PixelChange {
            layer_index,
            x,
            y,
            old_color,
            ..
        } => {
            if let Some(layer) = state.layers.get_mut(layer_index) {
                layer.set_pixel(x, y, old_color);
            }
        }
        state::EditCommand::MultiPixelChange {
            layer_index,
            changes,
        } => {
            if let Some(layer) = state.layers.get_mut(layer_index) {
                for (x, y, old_color, _) in changes {
                    layer.set_pixel(x, y, old_color);
                }
            }
        }
    }
}

fn apply_redo_command(state: &mut EditorState, command: state::EditCommand) {
    match command {
        state::EditCommand::PixelChange {
            layer_index,
            x,
            y,
            new_color,
            ..
        } => {
            if let Some(layer) = state.layers.get_mut(layer_index) {
                layer.set_pixel(x, y, new_color);
            }
        }
        state::EditCommand::MultiPixelChange {
            layer_index,
            changes,
        } => {
            if let Some(layer) = state.layers.get_mut(layer_index) {
                for (x, y, _, new_color) in changes {
                    layer.set_pixel(x, y, new_color);
                }
            }
        }
    }
}

fn view(state: &EditorState) -> iced::Element<'_, Message> {
    ui::view(state)
}
