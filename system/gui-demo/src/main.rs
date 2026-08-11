//! Host-side compositor demo: desktop + taskbar + terminal window.
//!
//! Writes `target/aether-desktop-demo.ppm` and prints a short summary.

use std::fs::File;
use std::io::{self, Write};
use std::path::PathBuf;

use aether_compositor::Compositor;
use aether_desktop::DesktopShell;
use aether_gui_ipc::{CreateWindowRequest, IpcMessage, MessageHeader, MessageKind};
use aether_terminal::Terminal;
use aether_window::{pack_rgba8888, Rect, WindowId};
use embedded_graphics::pixelcolor::Rgb888;

fn main() -> io::Result<()> {
    const WIDTH: u32 = 800;
    const HEIGHT: u32 = 600;

    let mut compositor = Compositor::new(WIDTH, HEIGHT);
    let mut shell = DesktopShell::new();
    shell.launcher_mut().register("Terminal", "aether-terminal");
    shell.launcher_mut().register("Files", "aether-files");

    let work = shell.work_area(WIDTH, HEIGHT);

    // Simulate a client requesting a window over IPC.
    compositor.ipc().client_send(IpcMessage::CreateWindow {
        header: MessageHeader::new(MessageKind::CreateWindow, WindowId(0)),
        body: CreateWindowRequest {
            title: "Terminal".into(),
            bounds: Rect { x: 48, y: 48, width: 480, height: 320 },
        },
    });
    compositor.poll_ipc().expect("ipc");

    let terminal_id =
        compositor.add_window("Terminal", Rect { x: 48, y: 72, width: 480, height: 320 });

    if let Some(window) = compositor.window_mut(terminal_id) {
        let mut term = Terminal::new(window.surface().width(), window.surface().height());
        term.writeln("Aether OS — GUI foundation demo (M10)");
        term.writeln("Compositor: aether-compositor");
        term.writeln("Desktop shell: aether-desktop");
        term.writeln("");
        term.writeln("aether@desktop:~$ echo hello");
        term.writeln("hello");
        term.render().expect("terminal render");

        let pixels = term.surface().pixels().to_vec();
        window.surface_mut().pixels_mut().copy_from_slice(&pixels);
    }

    let notes_id = compositor.add_window("Notes", Rect { x: 560, y: 96, width: 200, height: 160 });
    if let Some(window) = compositor.window_mut(notes_id) {
        window.surface_mut().clear(Rgb888::new(0xEB, 0xCB, 0x8B));
    }

    compositor.render().expect("composite");
    compositor.decorate_window(terminal_id).expect("decorate terminal");
    compositor.decorate_window(notes_id).expect("decorate notes");
    shell.render(&mut compositor).expect("shell");

    let output = demo_output_path();
    write_ppm(&output, compositor.display())?;

    println!("Aether compositor demo");
    println!("  display: {WIDTH}x{HEIGHT}");
    println!("  windows: {}", compositor.windows_by_z_order().len());
    println!("  work area: {}x{}", work.width, work.height);
    println!("  wrote: {}", output.display());
    Ok(())
}

fn demo_output_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("target")
        .join("aether-desktop-demo.ppm")
}

fn write_ppm(path: &std::path::Path, display: &aether_window::SurfaceBuffer) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut file = File::create(path)?;
    writeln!(file, "P6")?;
    writeln!(file, "{} {}", display.width(), display.height())?;
    writeln!(file, "255")?;

    for &pixel in display.pixels() {
        let packed = pack_rgba8888(Rgb888::new(
            (pixel & 0xFF) as u8,
            ((pixel >> 8) & 0xFF) as u8,
            ((pixel >> 16) & 0xFF) as u8,
        ));
        file.write_all(&[
            (packed & 0xFF) as u8,
            ((packed >> 8) & 0xFF) as u8,
            ((packed >> 16) & 0xFF) as u8,
        ])?;
    }

    Ok(())
}
