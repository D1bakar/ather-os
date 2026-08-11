//! Minimal compositor prototype for Aether OS.
//!
//! Manages a display-sized back buffer, stacks windows, and exposes a host-side
//! IPC endpoint for future userspace clients.

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]
#![deny(missing_docs)]

extern crate alloc;

use alloc::vec::Vec;
use core::fmt;

use embedded_graphics::geometry::{Point, Size};
use embedded_graphics::pixelcolor::Rgb888;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{PrimitiveStyle, Rectangle};

use aether_gui_ipc::{
    CreateWindowRequest, IpcEndpoint, IpcMessage, MessageHeader, MessageKind, SharedSurfaceRegion,
};
use aether_window::{unpack_rgba8888, Rect, SurfaceBuffer, Window, WindowId};

/// Default desktop background color.
pub const DESKTOP_BG: Rgb888 = Rgb888::new(0x1E, 0x27, 0x36);

/// Compositor error type.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompositorError {
    /// Window id was not found.
    WindowNotFound,
    /// IPC message could not be handled.
    InvalidIpc,
    /// A draw operation failed.
    RenderFailed,
}

impl fmt::Display for CompositorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WindowNotFound => write!(f, "window not found"),
            Self::InvalidIpc => write!(f, "invalid IPC message"),
            Self::RenderFailed => write!(f, "render failed"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for CompositorError {}

/// Scene graph and display back-buffer manager.
#[derive(Debug)]
pub struct Compositor {
    display: SurfaceBuffer,
    windows: Vec<Window>,
    next_window_id: u32,
    ipc: IpcEndpoint,
}

impl Compositor {
    /// Creates a compositor targeting a display of the given size.
    #[must_use]
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            display: SurfaceBuffer::new(width, height),
            windows: Vec::new(),
            next_window_id: 1,
            ipc: IpcEndpoint::new(),
        }
    }

    /// Display width in pixels.
    #[must_use]
    pub fn width(&self) -> u32 {
        self.display.width()
    }

    /// Display height in pixels.
    #[must_use]
    pub fn height(&self) -> u32 {
        self.display.height()
    }

    /// Shared IPC endpoint used by clients in the host prototype.
    pub fn ipc(&mut self) -> &mut IpcEndpoint {
        &mut self.ipc
    }

    /// Adds a window directly (compositor-internal API).
    pub fn add_window(
        &mut self,
        title: impl Into<alloc::string::String>,
        bounds: Rect,
    ) -> WindowId {
        let id = WindowId(self.next_window_id);
        self.next_window_id = self.next_window_id.saturating_add(1);
        self.windows.push(Window::new(id, title, bounds));
        id
    }

    /// Returns an immutable reference to a window.
    #[must_use]
    pub fn window(&self, id: WindowId) -> Option<&Window> {
        self.windows.iter().find(|w| w.id() == id)
    }

    /// Returns a mutable reference to a window.
    pub fn window_mut(&mut self, id: WindowId) -> Option<&mut Window> {
        self.windows.iter_mut().find(|w| w.id() == id)
    }

    /// All managed windows sorted by ascending z-order.
    #[must_use]
    pub fn windows_by_z_order(&self) -> Vec<&Window> {
        let mut ordered: Vec<&Window> = self.windows.iter().collect();
        ordered.sort_by_key(|w| w.z_order());
        ordered
    }

    /// Processes pending IPC messages from clients.
    pub fn poll_ipc(&mut self) -> Result<(), CompositorError> {
        while let Some(message) = self.ipc.compositor_recv() {
            self.handle_ipc(message)?;
        }
        Ok(())
    }

    fn handle_ipc(&mut self, message: IpcMessage) -> Result<(), CompositorError> {
        match message {
            IpcMessage::CreateWindow { body, .. } => {
                self.create_window_from_ipc(body)?;
            }
            IpcMessage::Damage { header, .. } => {
                let _ = header;
                // Repaint is triggered by the next `render()` call in this prototype.
            }
            IpcMessage::Ping { header } => {
                self.ipc.compositor_send(IpcMessage::Pong { header });
            }
            _ => return Err(CompositorError::InvalidIpc),
        }
        Ok(())
    }

    fn create_window_from_ipc(
        &mut self,
        request: CreateWindowRequest,
    ) -> Result<WindowId, CompositorError> {
        let id = self.add_window(request.title, request.bounds);
        let bounds = self.window(id).ok_or(CompositorError::WindowNotFound)?.bounds();
        let buffer_id = self.ipc.alloc_buffer_id();
        let region =
            SharedSurfaceRegion::new(buffer_id, bounds.width, bounds.height, bounds.width * 4);
        self.ipc.compositor_send(IpcMessage::SurfaceCreated {
            header: MessageHeader::new(MessageKind::SurfaceCreated, id),
            region,
        });
        Ok(id)
    }

    /// Clears the display and composites all visible windows.
    pub fn render(&mut self) -> Result<(), CompositorError> {
        self.display.clear(DESKTOP_BG);

        let ordered: Vec<WindowId> = self
            .windows_by_z_order()
            .into_iter()
            .filter(|w| w.is_visible())
            .map(|w| w.id())
            .collect();

        for id in ordered {
            self.composite_window(id)?;
        }

        Ok(())
    }

    fn composite_window(&mut self, id: WindowId) -> Result<(), CompositorError> {
        let Some(window) = self.windows.iter().find(|w| w.id() == id) else {
            return Err(CompositorError::WindowNotFound);
        };

        let bounds = window.bounds();
        let src = window.surface().pixels().to_vec();
        let display_width = self.display.width() as usize;
        let display_height = self.display.height() as usize;
        let dst = self.display.pixels_mut();

        for row in 0..bounds.height as usize {
            let dst_y = bounds.y as usize + row;
            if dst_y >= display_height {
                break;
            }
            for col in 0..bounds.width as usize {
                let dst_x = bounds.x as usize + col;
                if dst_x >= display_width {
                    break;
                }
                let src_index = row * bounds.width as usize + col;
                let dst_index = dst_y * display_width + dst_x;
                let color = unpack_rgba8888(src[src_index]);
                dst[dst_index] = aether_window::pack_rgba8888(color);
            }
        }

        Ok(())
    }

    /// Draws a solid rectangle directly on the display (used by desktop shell).
    pub fn fill_display_rect(&mut self, rect: Rect, color: Rgb888) -> Result<(), CompositorError> {
        self.display.fill_rect(rect, color).map_err(|_| CompositorError::RenderFailed)
    }

    /// Returns the composited display buffer.
    #[must_use]
    pub fn display(&self) -> &SurfaceBuffer {
        &self.display
    }

    /// Returns the composited display buffer mutably.
    pub fn display_mut(&mut self) -> &mut SurfaceBuffer {
        &mut self.display
    }

    /// Draws a simple window decoration frame around `id`.
    pub fn decorate_window(&mut self, id: WindowId) -> Result<(), CompositorError> {
        let Some(window) = self.windows.iter().find(|w| w.id() == id) else {
            return Err(CompositorError::WindowNotFound);
        };
        let bounds = window.bounds();
        let frame = Rect {
            x: bounds.x,
            y: bounds.y.saturating_sub(24),
            width: bounds.width,
            height: bounds.height.saturating_add(24),
        };

        let style = PrimitiveStyle::with_fill(Rgb888::new(0x2D, 0x3A, 0x4F));
        Rectangle::new(Point::new(frame.x, frame.y), Size::new(frame.width, frame.height))
            .into_styled(style)
            .draw(&mut self.display.draw_target())
            .map_err(|_| CompositorError::RenderFailed)?;

        let title_bar = Rect { x: frame.x, y: frame.y, width: frame.width, height: 24 };
        self.fill_display_rect(title_bar, Rgb888::new(0x3B, 0x4F, 0x6B))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compositor_renders_empty_desktop() {
        let mut compositor = Compositor::new(640, 480);
        compositor.render().expect("render");
        assert_eq!(compositor.display().pixels().len(), 640 * 480);
    }
}
