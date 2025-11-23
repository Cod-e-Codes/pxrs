# pxrs

Pixel art editor built with Rust and Iced.

## Features

- Tools: Pencil, Eraser, Fill, Selection, Eyedropper
- Layers: Multiple layers with visibility toggle and opacity control
- Undo/Redo: History system with up to 100 commands
- File Operations: Open and save PNG, GIF, BMP formats
- View: Zoom (1x-32x), grid display toggle
- Selection: Rectangular selection with copy/paste/cut
- Mirror: Horizontal and vertical mirror modes
- Colors: Primary/secondary color selection, eyedropper, used colors history (up to 32)
- Brush: Adjustable size (1-20 pixels)

## Keyboard Shortcuts

- Ctrl+Z: Undo
- Ctrl+Shift+Z / Ctrl+Y: Redo
- Ctrl+C: Copy selection
- Ctrl+V: Paste selection
- Ctrl+X: Cut selection
- Ctrl+A: Select all
- Delete/Backspace: Clear selection

## Requirements

- Rust (edition 2024)
- Dependencies: iced (0.13.1 with canvas feature), image (0.25.9), rfd (0.15.4), bytemuck (1.24.0)

## Building

```bash
cargo build --release
```

## Usage

Run the application:

```bash
cargo run
```

The editor starts with a 32x32 pixel canvas. Use the toolbar to select tools and adjust settings. Open images via File menu or create new files. Save your work in PNG, GIF, or BMP format.

