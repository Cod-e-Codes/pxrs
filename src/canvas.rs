use crate::message::Message;
use crate::state::EditorState;
use iced::mouse;
use iced::widget::canvas;
use iced::{Color, Point, Rectangle, Size};

pub struct CanvasProgram {
    state: EditorState,
}

impl CanvasProgram {
    pub fn new(state: EditorState) -> Self {
        Self { state }
    }

    pub fn update_state(&mut self, state: EditorState) {
        self.state = state;
    }

    fn canvas_to_pixel(&self, point: Point, bounds: Rectangle, zoom: f32) -> Option<(u32, u32)> {
        // Calculate pixel coordinates from canvas coordinates
        let pixel_size = zoom;
        let canvas_pixel_width = self.state.canvas_width as f32 * pixel_size;
        let canvas_pixel_height = self.state.canvas_height as f32 * pixel_size;

        // Calculate center offsets to center the canvas in the bounds
        let offset_x = (bounds.width - canvas_pixel_width) / 2.0;
        let offset_y = (bounds.height - canvas_pixel_height) / 2.0;

        // Convert mouse position relative to canvas bounds
        // Note: point is already relative to bounds (from cursor.position_in(bounds))
        let x = point.x - offset_x;
        let y = point.y - offset_y;

        if x < 0.0 || y < 0.0 {
            return None;
        }

        let pixel_x = (x / pixel_size) as u32;
        let pixel_y = (y / pixel_size) as u32;

        if pixel_x < self.state.canvas_width && pixel_y < self.state.canvas_height {
            Some((pixel_x, pixel_y))
        } else {
            None
        }
    }
}

impl canvas::Program<Message> for CanvasProgram {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let zoom = self.state.zoom_level;
        let pixel_size = zoom;
        let canvas_pixel_width = self.state.canvas_width as f32 * pixel_size;
        let canvas_pixel_height = self.state.canvas_height as f32 * pixel_size;

        // Calculate center offsets to center the canvas in the bounds
        let offset_x = (bounds.width - canvas_pixel_width) / 2.0;
        let offset_y = (bounds.height - canvas_pixel_height) / 2.0;

        // Draw background checkerboard pattern
        let checker_size = 8.0;
        for y in 0..(bounds.height as u32 / checker_size as u32 + 1) {
            for x in 0..(bounds.width as u32 / checker_size as u32 + 1) {
                let is_light = (x + y) % 2 == 0;
                let color = if is_light {
                    Color::from_rgb(0.9, 0.9, 0.9)
                } else {
                    Color::from_rgb(0.8, 0.8, 0.8)
                };
                let point = Point::new(x as f32 * checker_size, y as f32 * checker_size);
                let size = Size::new(checker_size, checker_size);
                frame.fill_rectangle(point, size, canvas::Fill::from(color));
            }
        }

        // Draw all visible layers
        for layer in &self.state.layers {
            if !layer.visible {
                continue;
            }

            for y in 0..self.state.canvas_height {
                for x in 0..self.state.canvas_width {
                    let mut color = layer.get_pixel(x, y);
                    // Apply layer opacity to the color's alpha channel
                    color = Color::from_rgba(color.r, color.g, color.b, color.a * layer.opacity);
                    if color.a > 0.0 {
                        let point = Point::new(
                            offset_x + x as f32 * pixel_size,
                            offset_y + y as f32 * pixel_size,
                        );
                        let size = Size::new(pixel_size, pixel_size);
                        frame.fill_rectangle(point, size, canvas::Fill::from(color));
                    }
                }
            }
        }

        // Draw grid if enabled
        if self.state.grid_visible && zoom >= 4.0 {
            let grid_color = Color::from_rgba(0.5, 0.5, 0.5, 0.3);
            for x in 0..=self.state.canvas_width {
                let line_x = offset_x + x as f32 * pixel_size;
                frame.stroke(
                    &canvas::Path::line(
                        Point::new(line_x, offset_y),
                        Point::new(line_x, offset_y + canvas_pixel_height),
                    ),
                    canvas::Stroke::default()
                        .with_width(1.0)
                        .with_color(grid_color),
                );
            }
            for y in 0..=self.state.canvas_height {
                let line_y = offset_y + y as f32 * pixel_size;
                frame.stroke(
                    &canvas::Path::line(
                        Point::new(offset_x, line_y),
                        Point::new(offset_x + canvas_pixel_width, line_y),
                    ),
                    canvas::Stroke::default()
                        .with_width(1.0)
                        .with_color(grid_color),
                );
            }
        }

        // Draw selection rectangle if active
        if let Some(selection) = self.state.selection {
            let sel_x = offset_x + selection.x * pixel_size;
            let sel_y = offset_y + selection.y * pixel_size;
            let sel_width = selection.width * pixel_size;
            let sel_height = selection.height * pixel_size;

            // Draw selection border
            let sel_point = Point::new(sel_x, sel_y);
            let sel_size = Size::new(sel_width, sel_height);
            frame.stroke(
                &canvas::Path::rectangle(sel_point, sel_size),
                canvas::Stroke::default()
                    .with_width(2.0)
                    .with_color(Color::from_rgba(0.0, 0.5, 1.0, 1.0)),
            );

            // Draw selection overlay
            let overlay_color = Color::from_rgba(0.0, 0.5, 1.0, 0.2);
            frame.fill_rectangle(sel_point, sel_size, canvas::Fill::from(overlay_color));
        }

        vec![frame.into_geometry()]
    }

    fn update(
        &self,
        _state: &mut (),
        event: canvas::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> (canvas::event::Status, Option<Message>) {
        use canvas::Event;
        use mouse::Button;

        let position = match cursor.position_in(bounds) {
            Some(pos) => pos,
            None => return (canvas::event::Status::Ignored, None),
        };

        match event {
            Event::Mouse(mouse_event) => match mouse_event {
                mouse::Event::ButtonPressed(Button::Left) => {
                    if let Some((x, y)) =
                        self.canvas_to_pixel(position, bounds, self.state.zoom_level)
                    {
                        return (
                            canvas::event::Status::Captured,
                            Some(Message::DrawingStarted { x, y }),
                        );
                    }
                }
                mouse::Event::ButtonReleased(Button::Left) => {
                    if self.state.current_tool == crate::state::Tool::Selection
                        && self.state.selection.is_some()
                        && self.state.is_selecting
                    {
                        return (
                            canvas::event::Status::Captured,
                            Some(Message::SelectionEnded),
                        );
                    }
                    return (canvas::event::Status::Captured, Some(Message::DrawingEnded));
                }
                mouse::Event::CursorMoved { .. } => {
                    if let Some((x, y)) =
                        self.canvas_to_pixel(position, bounds, self.state.zoom_level)
                    {
                        if self.state.is_drawing {
                            return (
                                canvas::event::Status::Captured,
                                Some(Message::PixelDrawn { x, y }),
                            );
                        } else if self.state.current_tool == crate::state::Tool::Selection
                            && self.state.selection.is_some()
                            && self.state.is_selecting
                        {
                            // Update selection while dragging
                            return (
                                canvas::event::Status::Captured,
                                Some(Message::SelectionUpdated {
                                    x: x as f32,
                                    y: y as f32,
                                }),
                            );
                        }
                    }
                }
                mouse::Event::WheelScrolled { delta } => {
                    use mouse::ScrollDelta;
                    match delta {
                        ScrollDelta::Lines { y, .. } | ScrollDelta::Pixels { y, .. } => {
                            if y > 0.0 {
                                return (canvas::event::Status::Captured, Some(Message::ZoomIn));
                            } else if y < 0.0 {
                                return (canvas::event::Status::Captured, Some(Message::ZoomOut));
                            }
                        }
                    }
                }
                mouse::Event::ButtonPressed(Button::Middle) => {
                    // Start panning with middle mouse button
                    if let Some((x, y)) =
                        self.canvas_to_pixel(position, bounds, self.state.zoom_level)
                    {
                        return (
                            canvas::event::Status::Captured,
                            Some(Message::PanChanged {
                                x: x as f32,
                                y: y as f32,
                            }),
                        );
                    }
                }
                _ => {}
            },
            Event::Touch(_) => {
                // Touch events not handled
            }
            Event::Keyboard(_) => {
                // Forward keyboard events
                return (
                    canvas::event::Status::Ignored,
                    Some(Message::CanvasEvent(event)),
                );
            }
        }

        (canvas::event::Status::Ignored, None)
    }
}
