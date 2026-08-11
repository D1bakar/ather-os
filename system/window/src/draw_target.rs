//! `embedded-graphics` draw target adapter for [`SurfaceBuffer`](super::SurfaceBuffer).

use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::{OriginDimensions, Size};
use embedded_graphics::pixelcolor::Rgb888;
use embedded_graphics::prelude::Pixel;

use crate::{pack_rgba8888, SurfaceBuffer, SurfaceError};

/// Draw target wrapping a [`SurfaceBuffer`].
pub struct SurfaceDrawTarget<'a> {
    surface: &'a mut SurfaceBuffer,
}

impl<'a> SurfaceDrawTarget<'a> {
    /// Creates a draw target over `surface`.
    pub fn new(surface: &'a mut SurfaceBuffer) -> Self {
        Self { surface }
    }
}

impl OriginDimensions for SurfaceDrawTarget<'_> {
    fn size(&self) -> Size {
        Size::new(self.surface.width(), self.surface.height())
    }
}

impl DrawTarget for SurfaceDrawTarget<'_> {
    type Color = Rgb888;
    type Error = SurfaceError;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        let width = self.surface.width() as i32;
        let height = self.surface.height() as i32;

        for Pixel(coord, color) in pixels {
            if coord.x < 0 || coord.y < 0 || coord.x >= width || coord.y >= height {
                continue;
            }
            let index = coord.y as usize * self.surface.width() as usize + coord.x as usize;
            self.surface.pixels_mut()[index] = pack_rgba8888(color);
        }

        Ok(())
    }
}
