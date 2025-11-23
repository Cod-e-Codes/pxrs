use crate::canvas::CanvasProgram;
use crate::message::{ExportFormat, Message};
use crate::state::{EditorState, Tool};
use iced::widget;
use iced::{Alignment, Color, Element, Length};

pub fn view(state: &EditorState) -> Element<'_, Message> {
    let mut canvas_program = CanvasProgram::new(state.clone());
    canvas_program.update_state(state.clone());

    widget::column![
        // Top toolbar
        toolbar(state),
        // Main content area
        widget::row![
            // Left sidebar
            left_sidebar(state),
            // Canvas area
            widget::container(
                iced::widget::canvas(canvas_program)
                    .width(Length::Fill)
                    .height(Length::Fill)
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .style(widget::container::rounded_box),
            // Right sidebar
            right_sidebar(state),
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .spacing(10)
        .padding(10),
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn toolbar(state: &EditorState) -> Element<'_, Message> {
    widget::row![
        widget::button("New").on_press(Message::FileNew),
        widget::button("Open").on_press(Message::FileOpen),
        widget::button("Save").on_press(Message::FileSave),
        widget::pick_list(
            [ExportFormat::Png, ExportFormat::Gif, ExportFormat::Bmp].as_slice(),
            Some(state.selected_export_format),
            Message::ExportFormatSelected,
        ),
        widget::horizontal_space(),
        widget::text(format!("Zoom: {:.0}%", state.zoom_level * 100.0 / 8.0)),
        widget::slider(1.0..=32.0, state.zoom_level, Message::ZoomChanged),
        widget::button("+").on_press(Message::ZoomIn),
        widget::button("-").on_press(Message::ZoomOut),
    ]
    .spacing(10)
    .padding(10)
    .align_y(Alignment::Center)
    .into()
}

fn left_sidebar(state: &EditorState) -> Element<'_, Message> {
    widget::container(widget::scrollable(
        widget::column![
            widget::text("Tools").size(16),
            tool_buttons(state),
            widget::horizontal_rule(10),
            widget::text("Brush Size").size(16),
            brush_size_control(state),
            widget::horizontal_rule(10),
            widget::text("Color").size(16),
            color_picker(state),
            widget::horizontal_rule(10),
            widget::text("Layers").size(16),
            layer_list(state),
        ]
        .spacing(10)
        .padding(iced::Padding::new(10.0).right(20.0)),
    ))
    .width(Length::Fixed(200.0))
    .into()
}

fn tool_buttons(state: &EditorState) -> Element<'_, Message> {
    widget::column![
        widget::button(if state.current_tool == Tool::Pencil {
            "[P] Pencil"
        } else {
            "Pencil"
        })
        .on_press(Message::ToolSelected(Tool::Pencil)),
        widget::button(if state.current_tool == Tool::Eraser {
            "[E] Eraser"
        } else {
            "Eraser"
        })
        .on_press(Message::ToolSelected(Tool::Eraser)),
        widget::button(if state.current_tool == Tool::Fill {
            "[F] Fill"
        } else {
            "Fill"
        })
        .on_press(Message::ToolSelected(Tool::Fill)),
        widget::button(if state.current_tool == Tool::Selection {
            "[S] Select"
        } else {
            "Select"
        })
        .on_press(Message::ToolSelected(Tool::Selection)),
        widget::button(if state.current_tool == Tool::Eyedropper {
            "[I] Eyedropper"
        } else {
            "Eyedropper"
        })
        .on_press(Message::ToolSelected(Tool::Eyedropper)),
    ]
    .spacing(5)
    .into()
}

fn brush_size_control(state: &EditorState) -> Element<'_, Message> {
    widget::column![
        widget::row![
            widget::text("Size:"),
            widget::horizontal_space(),
            widget::text(format!("{}px", state.brush_size)),
        ]
        .spacing(5)
        .align_y(Alignment::Center),
        widget::slider(1.0..=20.0, state.brush_size as f32, |v| {
            Message::BrushSizeChanged(v as u32)
        }),
    ]
    .spacing(5)
    .into()
}

fn color_picker(state: &EditorState) -> Element<'_, Message> {
    let rgba = state.primary_color.into_rgba8();
    let r = rgba[0];
    let g = rgba[1];
    let b = rgba[2];

    let sec_rgba = state.secondary_color.into_rgba8();
    let sec_r = sec_rgba[0];
    let sec_g = sec_rgba[1];
    let sec_b = sec_rgba[2];

    widget::column![
        // Primary color preview (clickable to pick color)
        widget::text("Primary"),
        widget::button(
            widget::container(
                widget::text("")
                    .width(Length::Fill)
                    .height(Length::Fixed(50.0))
            )
            .style(|_theme| {
                widget::container::Style {
                    background: Some(state.primary_color.into()),
                    border: iced::border::Border {
                        radius: iced::border::Radius::from(5.0),
                        width: 1.0,
                        color: Color::BLACK,
                    },
                    ..Default::default()
                }
            })
            .width(Length::Fill)
            .height(Length::Fixed(50.0))
        )
        .on_press(Message::ColorPicked(state.primary_color)),
        // RGB sliders
        widget::text("Red"),
        widget::slider(0.0..=255.0, r as f32, move |v| {
            let rgba = state.primary_color.into_rgba8();
            Message::PrimaryColorChanged(Color::from_rgba8(
                v as u8,
                rgba[1],
                rgba[2],
                rgba[3] as f32 / 255.0,
            ))
        }),
        widget::text("Green"),
        widget::slider(0.0..=255.0, g as f32, move |v| {
            let rgba = state.primary_color.into_rgba8();
            Message::PrimaryColorChanged(Color::from_rgba8(
                rgba[0],
                v as u8,
                rgba[2],
                rgba[3] as f32 / 255.0,
            ))
        }),
        widget::text("Blue"),
        widget::slider(0.0..=255.0, b as f32, move |v| {
            let rgba = state.primary_color.into_rgba8();
            Message::PrimaryColorChanged(Color::from_rgba8(
                rgba[0],
                rgba[1],
                v as u8,
                rgba[3] as f32 / 255.0,
            ))
        }),
        widget::horizontal_rule(5),
        // Secondary color preview
        widget::text("Secondary"),
        widget::container(
            widget::text("")
                .width(Length::Fill)
                .height(Length::Fixed(50.0))
        )
        .style(|_theme| {
            widget::container::Style {
                background: Some(state.secondary_color.into()),
                border: iced::border::Border {
                    radius: iced::border::Radius::from(5.0),
                    width: 1.0,
                    color: Color::BLACK,
                },
                ..Default::default()
            }
        })
        .width(Length::Fill)
        .height(Length::Fixed(50.0)),
        widget::text("Red"),
        widget::slider(0.0..=255.0, sec_r as f32, move |v| {
            let rgba = state.secondary_color.into_rgba8();
            Message::SecondaryColorChanged(Color::from_rgba8(
                v as u8,
                rgba[1],
                rgba[2],
                rgba[3] as f32 / 255.0,
            ))
        }),
        widget::text("Green"),
        widget::slider(0.0..=255.0, sec_g as f32, move |v| {
            let rgba = state.secondary_color.into_rgba8();
            Message::SecondaryColorChanged(Color::from_rgba8(
                rgba[0],
                v as u8,
                rgba[2],
                rgba[3] as f32 / 255.0,
            ))
        }),
        widget::text("Blue"),
        widget::slider(0.0..=255.0, sec_b as f32, move |v| {
            let rgba = state.secondary_color.into_rgba8();
            Message::SecondaryColorChanged(Color::from_rgba8(
                rgba[0],
                rgba[1],
                v as u8,
                rgba[3] as f32 / 255.0,
            ))
        }),
    ]
    .spacing(5)
    .into()
}

fn layer_list(state: &EditorState) -> Element<'_, Message> {
    let mut layer_widgets: Vec<Element<Message>> = Vec::new();

    for (index, layer) in state.layers.iter().enumerate().rev() {
        let is_active = index == state.active_layer_index;
        let layer_opacity = layer.opacity;
        let layer_index = index;

        let opacity_slider = widget::slider(0.0..=1.0, layer_opacity, move |v| {
            Message::LayerOpacityChanged {
                index: layer_index,
                opacity: v,
            }
        })
        .width(Length::Fill)
        .step(0.01);

        let layer_card = widget::container(
            widget::column![
                // First line: Checkbox and Layer name
                widget::row![
                    widget::checkbox("", layer.visible)
                        .on_toggle(move |_| Message::LayerVisibilityToggled(layer_index)),
                    widget::button(&*layer.name)
                        .on_press(Message::LayerSelected(layer_index))
                        .padding([4, 8])
                        .style(if is_active {
                            widget::button::primary
                        } else {
                            widget::button::text
                        }),
                ]
                .spacing(5)
                .align_y(Alignment::Center)
                .width(Length::Fill),
                // Second line: Action buttons
                widget::row![
                    widget::button("E").on_press(Message::LayerRenamed {
                        index: layer_index,
                        name: layer.name.clone(),
                    }),
                    widget::button("^").on_press(if layer_index > 0 {
                        Message::LayerMoved {
                            from: layer_index,
                            to: layer_index - 1,
                        }
                    } else {
                        Message::None
                    }),
                    widget::button("v").on_press(if layer_index < state.layers.len() - 1 {
                        Message::LayerMoved {
                            from: layer_index,
                            to: layer_index + 1,
                        }
                    } else {
                        Message::None
                    }),
                    if state.layers.len() > 1 {
                        widget::button("X")
                            .on_press(Message::LayerDeleted(layer_index))
                            .style(widget::button::danger)
                    } else {
                        widget::button("X").style(widget::button::secondary)
                    },
                    widget::horizontal_space(),
                ]
                .spacing(5)
                .align_y(Alignment::Center)
                .width(Length::Fill),
                // Third line: Opacity slider
                widget::row![
                    widget::text("Opacity:").size(12),
                    opacity_slider,
                    widget::text(format!("{:.0}%", layer_opacity * 100.0))
                        .size(12)
                        .width(Length::Fixed(40.0)),
                ]
                .spacing(5)
                .align_y(Alignment::Center)
                .width(Length::Fill),
            ]
            .spacing(8)
            .width(Length::Fill),
        )
        .style(move |_theme| widget::container::Style {
            background: if is_active {
                Some(Color::from_rgba(0.1, 0.3, 0.6, 0.3).into())
            } else {
                Some(Color::from_rgba(0.2, 0.2, 0.2, 0.3).into())
            },
            border: iced::border::Border {
                radius: iced::border::Radius::from(5.0),
                width: if is_active { 2.0 } else { 1.0 },
                color: if is_active {
                    Color::from_rgba(0.2, 0.5, 0.9, 1.0)
                } else {
                    Color::from_rgba(0.4, 0.4, 0.4, 0.5)
                },
            },
            ..Default::default()
        })
        .padding(8)
        .width(Length::Fill);

        layer_widgets.push(layer_card.into());
    }

    widget::column![
        widget::column(layer_widgets).spacing(8),
        widget::button("+ Add Layer").on_press(Message::LayerAdded(format!(
            "Layer {}",
            state.layers.len() + 1
        ))),
    ]
    .spacing(8)
    .into()
}

fn right_sidebar(state: &EditorState) -> Element<'_, Message> {
    let mut used_colors_grid = widget::column![].spacing(5);

    // Create grid of used colors (4 per row)
    let mut current_row = widget::row![].spacing(5);
    for (i, color) in state.used_colors.iter().enumerate() {
        // Start a new row every 4 colors (after completing a full row)
        if i > 0 && i % 4 == 0 {
            used_colors_grid = used_colors_grid.push(current_row);
            current_row = widget::row![].spacing(5);
        }

        let color_button = widget::button(
            widget::container(widget::text(""))
                .width(Length::Fixed(30.0))
                .height(Length::Fixed(30.0))
                .style(move |_theme| widget::container::Style {
                    background: Some((*color).into()),
                    border: iced::border::Border {
                        radius: iced::border::Radius::from(3.0),
                        width: 1.0,
                        color: Color::BLACK,
                    },
                    ..Default::default()
                }),
        )
        .on_press(Message::UsedColorPicked(*color))
        .padding(0);

        current_row = current_row.push(color_button);
    }

    // Always add the last row if there are any colors (it will contain the remaining colors)
    if !state.used_colors.is_empty() {
        used_colors_grid = used_colors_grid.push(current_row);
    }

    widget::container(widget::scrollable(
        widget::column![
            widget::text("Properties").size(16),
            widget::horizontal_rule(10),
            widget::text("Used Colors").size(14),
            widget::scrollable(used_colors_grid).height(Length::Fixed(150.0)),
            widget::horizontal_rule(10),
            widget::text("Canvas Size"),
            widget::row![
                widget::text_input("Width", &state.canvas_width.to_string()).on_input(move |s| {
                    s.parse::<u32>()
                        .ok()
                        .map(|w| Message::CanvasResized {
                            width: w,
                            height: state.canvas_height,
                        })
                        .unwrap_or(Message::None)
                }),
                widget::text("x"),
                widget::text_input("Height", &state.canvas_height.to_string()).on_input(move |s| {
                    s.parse::<u32>()
                        .ok()
                        .map(|h| Message::CanvasResized {
                            width: state.canvas_width,
                            height: h,
                        })
                        .unwrap_or(Message::None)
                }),
            ]
            .spacing(5),
            widget::button("Clear Canvas").on_press(Message::CanvasCleared),
            widget::horizontal_rule(10),
            widget::text("Grid"),
            widget::toggler(state.grid_visible).on_toggle(|_| Message::GridToggled),
            widget::horizontal_rule(10),
            widget::text("Selection"),
            widget::button("Copy (Ctrl+C)").on_press(Message::CopySelection),
            widget::button("Cut (Ctrl+X)").on_press(Message::CutSelection),
            widget::button("Clear").on_press(Message::SelectionCleared),
            widget::horizontal_rule(10),
            widget::text("Mirror Mode"),
            widget::row![
                widget::text("Horizontal"),
                widget::horizontal_space(),
                widget::toggler(state.mirror_horizontal)
                    .on_toggle(|_| Message::MirrorHorizontalToggled),
            ]
            .spacing(5)
            .width(Length::Fill),
            widget::row![
                widget::text("Vertical"),
                widget::horizontal_space(),
                widget::toggler(state.mirror_vertical)
                    .on_toggle(|_| Message::MirrorVerticalToggled),
            ]
            .spacing(5)
            .width(Length::Fill),
        ]
        .spacing(10)
        .padding(iced::Padding::new(10.0).right(20.0)),
    ))
    .width(Length::Fixed(200.0))
    .into()
}
