use crate::state::Tool;
use iced::Color;

#[derive(Debug, Clone)]
pub enum Message {
    // Tool selection
    ToolSelected(Tool),

    // Color changes
    PrimaryColorChanged(Color),
    SecondaryColorChanged(Color),
    ColorPicked(Color),
    UsedColorPicked(Color),

    // Brush settings
    BrushSizeChanged(u32),

    // Canvas operations
    CanvasResized { width: u32, height: u32 },
    CanvasCleared,

    // Layer operations
    LayerAdded(String),
    LayerDeleted(usize),
    LayerMoved { from: usize, to: usize },
    LayerVisibilityToggled(usize),
    LayerSelected(usize),
    LayerOpacityChanged { index: usize, opacity: f32 },
    LayerRenamed { index: usize, name: String },

    // Drawing operations
    PixelDrawn { x: u32, y: u32 },
    DrawingStarted { x: u32, y: u32 },
    DrawingEnded,

    // File operations
    FileNew,
    FileOpen,
    FileSave,
    FileSaveDialogResult { path: String, format: ExportFormat },
    ExportFormatSelected(ExportFormat),
    FileLoaded { path: String, data: Vec<u8> },
    FileSaved { path: String },

    // Undo/Redo
    Undo,
    Redo,

    // View operations
    ZoomChanged(f32),
    ZoomIn,
    ZoomOut,
    GridToggled,
    PanChanged { x: f32, y: f32 },

    // Selection
    SelectionStarted { x: f32, y: f32 },
    SelectionUpdated { x: f32, y: f32 },
    SelectionEnded,
    SelectionCleared,
    CopySelection,
    PasteSelection { x: u32, y: u32 },
    CutSelection,

    // Canvas events
    CanvasEvent(iced::widget::canvas::Event),

    // Mirror mode
    MirrorHorizontalToggled,
    MirrorVerticalToggled,

    // No-op
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    Png,
    Gif,
    Bmp,
}

impl std::fmt::Display for ExportFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExportFormat::Png => write!(f, "PNG"),
            ExportFormat::Gif => write!(f, "GIF"),
            ExportFormat::Bmp => write!(f, "BMP"),
        }
    }
}
