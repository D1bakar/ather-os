//! Linear framebuffer access from [`BootInfo`] metadata.

#![no_std]
#![deny(missing_docs)]

use aether_types::FramebufferInfo;

/// Pixel format: 32-bit BGRA (UEFI-style, common in QEMU GOP).
pub const PIXEL_FORMAT_BGRA: u32 = 1;

/// Active linear framebuffer description.
#[derive(Clone, Copy, Debug)]
pub struct Framebuffer {
    base: *mut u8,
    width: u32,
    height: u32,
    stride: u32,
    pixel_format: u32,
}

impl Framebuffer {
    /// Probes and initializes a framebuffer from boot-loader metadata.
    ///
    /// Returns `None` when no valid framebuffer was provided.
    pub fn init(info: &FramebufferInfo) -> Option<Self> {
        if info.base == 0 || info.width == 0 || info.height == 0 {
            return None;
        }

        let stride = if info.stride != 0 { info.stride } else { info.width * 4 };

        Some(Self {
            base: info.base as *mut u8,
            width: info.width,
            height: info.height,
            stride,
            pixel_format: info.pixel_format,
        })
    }

    /// Returns `true` when the framebuffer is usable.
    #[must_use]
    pub const fn is_valid(&self) -> bool {
        !self.base.is_null() && self.width > 0 && self.height > 0
    }

    /// Framebuffer width in pixels.
    #[must_use]
    pub const fn width(&self) -> u32 {
        self.width
    }

    /// Framebuffer height in pixels.
    #[must_use]
    pub const fn height(&self) -> u32 {
        self.height
    }

    /// Bytes per scan line.
    #[must_use]
    pub const fn stride(&self) -> u32 {
        self.stride
    }

    /// Clears the screen to a 32-bit BGRA color.
    pub fn clear(&self, color: u32) {
        if !self.is_valid() {
            return;
        }

        for y in 0..self.height {
            for x in 0..self.width {
                self.put_pixel(x, y, color);
            }
        }
    }

    /// Writes one 32-bit pixel at `(x, y)`.
    pub fn put_pixel(&self, x: u32, y: u32, color: u32) {
        if x >= self.width || y >= self.height {
            return;
        }

        let offset = y as u64 * u64::from(self.stride) + u64::from(x * 4);
        // SAFETY: Boot loader mapped the framebuffer; identity-mapped in early boot.
        unsafe {
            let ptr = self.base.add(offset as usize) as *mut u32;
            ptr.write_volatile(color);
        }
    }
}

static mut ACTIVE: Option<Framebuffer> = None;

/// Initializes the global framebuffer from boot info.
pub fn init(info: &FramebufferInfo) -> bool {
    if let Some(fb) = Framebuffer::init(info) {
        // SAFETY: Single-threaded early boot.
        unsafe {
            ACTIVE = Some(fb);
        }
        true
    } else {
        false
    }
}

/// Returns a copy of the active framebuffer, if initialized.
pub fn get() -> Option<Framebuffer> {
    // SAFETY: Read-only access during early boot.
    unsafe { ACTIVE }
}
