//! Framebuffer-backed terminal emulator stub.
//!
//! Maintains a scrollable character grid and renders it into a [`SurfaceBuffer`]
//! using `embedded-graphics` monospace text.

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]
#![deny(missing_docs)]

extern crate alloc;

use alloc::vec::Vec;
use core::fmt;

use embedded_graphics::geometry::Point;
use embedded_graphics::mono_font::ascii::FONT_6X10;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::Rgb888;
use embedded_graphics::prelude::*;
use embedded_graphics::text::Text;

use aether_window::{Rect, SurfaceBuffer};

/// Default terminal foreground color.
pub const FG: Rgb888 = Rgb888::new(0xE5, 0xE9, 0xF0);

/// Default terminal background color.
pub const BG: Rgb888 = Rgb888::new(0x0F, 0x14, 0x1C);

/// Character cell width in pixels (matches [`FONT_6X10`] metrics).
pub const CELL_WIDTH: u32 = 6;

/// Character cell height in pixels.
pub const CELL_HEIGHT: u32 = 10;

/// Minimal VT-style terminal grid backed by a pixel surface.
#[derive(Clone, Debug)]
pub struct Terminal {
    cols: u32,
    rows: u32,
    buffer: Vec<u8>,
    cursor_col: u32,
    cursor_row: u32,
    surface: SurfaceBuffer,
}

impl Terminal {
    /// Creates a terminal sized to fit `width`×`height` pixels.
    #[must_use]
    pub fn new(width: u32, height: u32) -> Self {
        let cols = width / CELL_WIDTH;
        let rows = height / CELL_HEIGHT;
        let buffer = vec![b' '; (cols * rows) as usize];
        let surface = SurfaceBuffer::new(width, height);
        Self { cols, rows, buffer, cursor_col: 0, cursor_row: 0, surface }
    }

    /// Grid width in columns.
    #[must_use]
    pub const fn cols(&self) -> u32 {
        self.cols
    }

    /// Grid height in rows.
    #[must_use]
    pub const fn rows(&self) -> u32 {
        self.rows
    }

    /// Current cursor column.
    #[must_use]
    pub const fn cursor_col(&self) -> u32 {
        self.cursor_col
    }

    /// Current cursor row.
    #[must_use]
    pub const fn cursor_row(&self) -> u32 {
        self.cursor_row
    }

    /// Underlying pixel surface.
    #[must_use]
    pub fn surface(&self) -> &SurfaceBuffer {
        &self.surface
    }

    /// Underlying pixel surface (mutable).
    pub fn surface_mut(&mut self) -> &mut SurfaceBuffer {
        &mut self.surface
    }

    /// Writes printable ASCII bytes and handles `\n`.
    pub fn write_bytes(&mut self, data: &[u8]) {
        for &byte in data {
            self.write_byte(byte);
        }
    }

    /// Writes a line of text followed by newline.
    pub fn writeln(&mut self, line: &str) {
        self.write_bytes(line.as_bytes());
        self.newline();
    }

    fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.newline(),
            0x20..=0x7E => {
                if self.cols > 0 && self.rows > 0 {
                    let index = (self.cursor_row * self.cols + self.cursor_col) as usize;
                    if index < self.buffer.len() {
                        self.buffer[index] = byte;
                    }
                    self.cursor_col = self.cursor_col.saturating_add(1);
                    if self.cursor_col >= self.cols {
                        self.newline();
                    }
                }
            }
            _ => {}
        }
    }

    fn newline(&mut self) {
        self.cursor_col = 0;
        if self.cursor_row + 1 >= self.rows {
            self.scroll_up();
        } else {
            self.cursor_row += 1;
        }
    }

    fn scroll_up(&mut self) {
        if self.cols == 0 || self.rows == 0 {
            return;
        }
        let row_len = self.cols as usize;
        self.buffer.copy_within(row_len.., 0);
        let start = self.buffer.len().saturating_sub(row_len);
        for cell in &mut self.buffer[start..] {
            *cell = b' ';
        }
    }

    /// Clears the grid and resets the cursor.
    pub fn clear(&mut self) {
        for cell in &mut self.buffer {
            *cell = b' ';
        }
        self.cursor_col = 0;
        self.cursor_row = 0;
    }

    /// Renders the character grid into the pixel surface.
    pub fn render(&mut self) -> Result<(), TerminalError> {
        self.surface.clear(BG);

        let style = MonoTextStyle::new(&FONT_6X10, FG);
        for row in 0..self.rows {
            let start = (row * self.cols) as usize;
            let end = start + self.cols as usize;
            let line = core::str::from_utf8(&self.buffer[start..end])
                .map_err(|_| TerminalError::InvalidUtf8)?;
            Text::new(line, Point::new(4, (row * CELL_HEIGHT + 10) as i32), style)
                .draw(&mut self.surface.draw_target())
                .map_err(|_| TerminalError::RenderFailed)?;
        }

        Ok(())
    }

    /// Clears the surface to the terminal background color.
    pub fn clear_surface(&mut self, surface: &mut SurfaceBuffer) {
        surface.clear(BG);
    }

    /// Copies the rendered terminal surface into `dest` at `bounds`.
    pub fn blit_to(&self, dest: &mut SurfaceBuffer, bounds: Rect) -> Result<(), TerminalError> {
        let src = self.surface.pixels();
        let dst_width = dest.width() as usize;
        let dst_height = dest.height() as usize;
        let dst = dest.pixels_mut();
        let src_width = self.surface.width() as usize;

        for row in 0..bounds.height as usize {
            let dst_y = bounds.y as usize + row;
            if dst_y >= dst_height {
                break;
            }
            for col in 0..bounds.width as usize {
                let dst_x = bounds.x as usize + col;
                if dst_x >= dst_width {
                    break;
                }
                if col >= src_width || row >= self.surface.height() as usize {
                    continue;
                }
                let src_index = row * src_width + col;
                let dst_index = dst_y * dst_width + dst_x;
                dst[dst_index] = src[src_index];
            }
        }

        Ok(())
    }
}

/// Terminal operation errors.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TerminalError {
    /// Grid contents were not valid UTF-8.
    InvalidUtf8,
    /// Rendering failed.
    RenderFailed,
}

impl fmt::Display for TerminalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidUtf8 => write!(f, "terminal grid is not valid UTF-8"),
            Self::RenderFailed => write!(f, "terminal render failed"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for TerminalError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terminal_writes_and_renders() {
        let mut term = Terminal::new(120, 60);
        term.writeln("ok");
        term.writeln("done");
        term.render().expect("render");
        assert_eq!(term.cursor_row(), 2);
    }
}
