//! Window and surface buffer primitives for the Aether OS compositor.
//!
//! This crate defines window geometry, pixel surfaces, and a [`DrawTarget`]
//! adapter so compositor and application code can render with `embedded-graphics`.

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]
#![deny(missing_docs)]

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

use embedded_graphics::geometry::{Point, Size};
use embedded_graphics::pixelcolor::{Rgb888, RgbColor};
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{PrimitiveStyle, Rectangle};

mod draw_target;

pub use draw_target::SurfaceDrawTarget;

/// Opaque identifier for a compositor-managed window.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct WindowId(pub u32);

impl fmt::Display for WindowId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "window-{}", self.0)
    }
}

/// Axis-aligned rectangle in pixel coordinates.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rect {
    /// Left edge (pixels).
    pub x: i32,
    /// Top edge (pixels).
    pub y: i32,
    /// Width (pixels).
    pub width: u32,
    /// Height (pixels).
    pub height: u32,
}

impl Rect {
    /// Returns a rectangle anchored at the origin with the given size.
    #[must_use]
    pub const fn from_size(width: u32, height: u32) -> Self {
        Self { x: 0, y: 0, width, height }
    }

    /// Returns `true` when the point lies inside the rectangle.
    #[must_use]
    pub fn contains(&self, point: Point) -> bool {
        point.x >= self.x
            && point.y >= self.y
            && point.x < self.x + self.width as i32
            && point.y < self.y + self.height as i32
    }
}

/// Pixel layout used by compositor surfaces.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PixelFormat {
    /// 32-bit RGBA with red in the least significant byte of each pixel word.
    Rgba8888,
}

/// A client-owned pixel buffer backing a window surface.
#[derive(Clone, Debug)]
pub struct SurfaceBuffer {
    width: u32,
    height: u32,
    format: PixelFormat,
    pixels: Vec<u32>,
}

impl SurfaceBuffer {
    /// Allocates a cleared surface of the given dimensions.
    #[must_use]
    pub fn new(width: u32, height: u32) -> Self {
        let len = (width as usize).saturating_mul(height as usize);
        Self { width, height, format: PixelFormat::Rgba8888, pixels: vec![0; len] }
    }

    /// Surface width in pixels.
    #[must_use]
    pub const fn width(&self) -> u32 {
        self.width
    }

    /// Surface height in pixels.
    #[must_use]
    pub const fn height(&self) -> u32 {
        self.height
    }

    /// Active pixel format.
    #[must_use]
    pub const fn format(&self) -> PixelFormat {
        self.format
    }

    /// Raw RGBA8888 pixel storage.
    #[must_use]
    pub fn pixels(&self) -> &[u32] {
        &self.pixels
    }

    /// Mutable raw RGBA8888 pixel storage.
    pub fn pixels_mut(&mut self) -> &mut [u32] {
        &mut self.pixels
    }

    /// Clears the surface to a solid color.
    pub fn clear(&mut self, color: Rgb888) {
        let packed = pack_rgba8888(color);
        for pixel in &mut self.pixels {
            *pixel = packed;
        }
    }

    /// Returns an `embedded-graphics` draw target over this surface.
    pub fn draw_target(&mut self) -> SurfaceDrawTarget<'_> {
        SurfaceDrawTarget::new(self)
    }

    /// Fills a rectangle relative to the surface origin.
    pub fn fill_rect(&mut self, rect: Rect, color: Rgb888) -> Result<(), SurfaceError> {
        let style = PrimitiveStyle::with_fill(color);
        Rectangle::new(Point::new(rect.x, rect.y), Size::new(rect.width, rect.height))
            .into_styled(style)
            .draw(&mut self.draw_target())
            .map_err(|_| SurfaceError::DrawFailed)
    }
}

/// Errors returned by surface operations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SurfaceError {
    /// A draw operation could not be completed.
    DrawFailed,
}

impl fmt::Display for SurfaceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DrawFailed => write!(f, "surface draw failed"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for SurfaceError {}

/// Compositor-managed top-level window.
#[derive(Clone, Debug)]
pub struct Window {
    id: WindowId,
    title: String,
    bounds: Rect,
    surface: SurfaceBuffer,
    visible: bool,
    z_order: i32,
}

impl Window {
    /// Creates a window with an empty surface sized to its bounds.
    #[must_use]
    pub fn new(id: WindowId, title: impl Into<String>, bounds: Rect) -> Self {
        let surface = SurfaceBuffer::new(bounds.width, bounds.height);
        Self { id, title: title.into(), bounds, surface, visible: true, z_order: 0 }
    }

    /// Window identifier.
    #[must_use]
    pub const fn id(&self) -> WindowId {
        self.id
    }

    /// Window title shown in the taskbar and decorations.
    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Screen-space bounds.
    #[must_use]
    pub const fn bounds(&self) -> Rect {
        self.bounds
    }

    /// Mutable client surface buffer.
    pub fn surface_mut(&mut self) -> &mut SurfaceBuffer {
        &mut self.surface
    }

    /// Client surface buffer.
    #[must_use]
    pub fn surface(&self) -> &SurfaceBuffer {
        &self.surface
    }

    /// Whether the compositor should include this window in the scene.
    #[must_use]
    pub const fn is_visible(&self) -> bool {
        self.visible
    }

    /// Updates visibility.
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    /// Stacking order; larger values paint above smaller ones.
    #[must_use]
    pub const fn z_order(&self) -> i32 {
        self.z_order
    }

    /// Updates stacking order.
    pub fn set_z_order(&mut self, z_order: i32) {
        self.z_order = z_order;
    }
}

/// Packs an [`Rgb888`] color into RGBA8888 storage (alpha = 255).
#[must_use]
pub fn pack_rgba8888(color: Rgb888) -> u32 {
    let r = color.r() as u32;
    let g = color.g() as u32;
    let b = color.b() as u32;
    (255 << 24) | (b << 16) | (g << 8) | r
}

/// Unpacks RGBA8888 storage into [`Rgb888`].
#[must_use]
pub const fn unpack_rgba8888(pixel: u32) -> Rgb888 {
    Rgb888::new((pixel & 0xFF) as u8, ((pixel >> 8) & 0xFF) as u8, ((pixel >> 16) & 0xFF) as u8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rect_contains_point() {
        let rect = Rect { x: 10, y: 20, width: 100, height: 50 };
        assert!(rect.contains(Point::new(10, 20)));
        assert!(!rect.contains(Point::new(9, 20)));
    }

    #[test]
    fn surface_clear_sets_pixels() {
        let mut surface = SurfaceBuffer::new(2, 2);
        surface.clear(Rgb888::new(0x11, 0x22, 0x33));
        assert!(surface
            .pixels()
            .iter()
            .all(|&p| p == pack_rgba8888(Rgb888::new(0x11, 0x22, 0x33))));
    }
}
