//! Desktop shell stubs: taskbar panel and application launcher.
//!
//! **Status:** prototype — draws a bottom panel and placeholder launcher affordance
//! on the compositor display buffer.

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]
#![deny(missing_docs)]

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use embedded_graphics::geometry::Point;
use embedded_graphics::mono_font::ascii::FONT_6X10;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::Rgb888;
use embedded_graphics::prelude::*;
use embedded_graphics::text::Text;

use aether_compositor::Compositor;
use aether_window::Rect;

/// Taskbar background color.
pub const TASKBAR_BG: Rgb888 = Rgb888::new(0x12, 0x18, 0x24);

/// Accent color for launcher and highlights.
pub const ACCENT: Rgb888 = Rgb888::new(0x5E, 0x81, 0xAC);

/// Bottom panel / taskbar configuration.
#[derive(Clone, Debug)]
pub struct Taskbar {
    height: u32,
}

impl Taskbar {
    /// Creates a taskbar with the given height in pixels.
    #[must_use]
    pub const fn new(height: u32) -> Self {
        Self { height }
    }

    /// Taskbar height.
    #[must_use]
    pub const fn height(&self) -> u32 {
        self.height
    }

    /// Paints the taskbar onto the compositor display.
    pub fn render(&self, compositor: &mut Compositor) -> Result<(), DesktopError> {
        let display_height = compositor.height();
        let y = display_height.saturating_sub(self.height) as i32;
        let rect = Rect { x: 0, y, width: compositor.width(), height: self.height };
        compositor.fill_display_rect(rect, TASKBAR_BG).map_err(|_| DesktopError::RenderFailed)?;

        let style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(0xD8, 0xDE, 0xE9));
        Text::new("Aether Desktop", Point::new(12, y + 18), style)
            .draw(&mut compositor.display_mut().draw_target())
            .map_err(|_| DesktopError::RenderFailed)?;

        Ok(())
    }
}

/// One entry in the launcher stub list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LauncherEntry {
    /// Display name shown in the launcher.
    pub label: String,
    /// Placeholder command or target identifier.
    pub command: String,
}

/// Application launcher stub.
#[derive(Clone, Debug, Default)]
pub struct Launcher {
    entries: Vec<LauncherEntry>,
}

impl Launcher {
    /// Creates an empty launcher.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a launcher entry.
    pub fn register(&mut self, label: impl Into<String>, command: impl Into<String>) {
        self.entries.push(LauncherEntry { label: label.into(), command: command.into() });
    }

    /// Registered entries.
    #[must_use]
    pub fn entries(&self) -> &[LauncherEntry] {
        &self.entries
    }

    /// Draws launcher buttons on the taskbar (stub — no click handling yet).
    pub fn render_on_taskbar(
        &self,
        compositor: &mut Compositor,
        taskbar: &Taskbar,
    ) -> Result<(), DesktopError> {
        let display_height = compositor.height();
        let y = display_height.saturating_sub(taskbar.height()) as i32;
        let mut x = 160i32;

        for entry in &self.entries {
            let button =
                Rect { x, y: y + 4, width: 88, height: taskbar.height().saturating_sub(8) };
            compositor.fill_display_rect(button, ACCENT).map_err(|_| DesktopError::RenderFailed)?;

            let style = MonoTextStyle::new(&FONT_6X10, Rgb888::new(0xE5, 0xE9, 0xF0));
            Text::new(entry.label.as_str(), Point::new(x + 8, y + 18), style)
                .draw(&mut compositor.display_mut().draw_target())
                .map_err(|_| DesktopError::RenderFailed)?;

            x += 96;
        }

        Ok(())
    }
}

/// Desktop shell coordinating taskbar and launcher.
#[derive(Clone, Debug)]
pub struct DesktopShell {
    taskbar: Taskbar,
    launcher: Launcher,
}

impl DesktopShell {
    /// Creates a shell with default taskbar height (40 px).
    #[must_use]
    pub fn new() -> Self {
        Self { taskbar: Taskbar::new(40), launcher: Launcher::new() }
    }

    /// Taskbar configuration.
    #[must_use]
    pub fn taskbar(&self) -> &Taskbar {
        &self.taskbar
    }

    /// Mutable launcher registry.
    pub fn launcher_mut(&mut self) -> &mut Launcher {
        &mut self.launcher
    }

    /// Renders the desktop chrome after the compositor scene graph.
    pub fn render(&self, compositor: &mut Compositor) -> Result<(), DesktopError> {
        self.taskbar.render(compositor)?;
        self.launcher.render_on_taskbar(compositor, &self.taskbar)?;
        Ok(())
    }

    /// Usable desktop area above the taskbar.
    #[must_use]
    pub fn work_area(&self, display_width: u32, display_height: u32) -> Rect {
        Rect {
            x: 0,
            y: 0,
            width: display_width,
            height: display_height.saturating_sub(self.taskbar.height()),
        }
    }
}

impl Default for DesktopShell {
    fn default() -> Self {
        Self::new()
    }
}

/// Desktop shell errors.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DesktopError {
    /// Rendering failed.
    RenderFailed,
}

impl core::fmt::Display for DesktopError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::RenderFailed => write!(f, "desktop render failed"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for DesktopError {}

#[cfg(test)]
mod tests {
    use super::*;
    use aether_compositor::Compositor;

    #[test]
    fn work_area_excludes_taskbar() {
        let shell = DesktopShell::new();
        let area = shell.work_area(800, 600);
        assert_eq!(area.height, 560);
    }

    #[test]
    fn shell_renders_on_compositor() {
        let mut compositor = Compositor::new(320, 240);
        compositor.render().expect("scene");
        let mut shell = DesktopShell::new();
        shell.launcher_mut().register("Terminal", "aether-terminal");
        shell.render(&mut compositor).expect("chrome");
    }
}
