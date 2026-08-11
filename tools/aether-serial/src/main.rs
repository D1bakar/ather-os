//! Host-side serial monitor for Aether OS development.

use std::env;
use std::fs::File;
use std::io::{self, BufRead, Read, Seek, SeekFrom, Write};
use std::net::{SocketAddr, TcpStream};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

const DEFAULT_LOG: &str = "build/qemu-serial.log";
const DEFAULT_TCP_PORT: u16 = 4444;
const POLL_MS: u64 = 100;

fn main() {
    if let Err(err) = run() {
        eprintln!("aether-serial: {err}");
        std::process::exit(1);
    }
}

fn run() -> io::Result<()> {
    let mut args = env::args().skip(1);
    let Some(command) = args.next() else {
        print_usage();
        return Ok(());
    };

    match command.as_str() {
        "follow" => {
            let file = args.next().map(PathBuf::from).unwrap_or_else(|| PathBuf::from(DEFAULT_LOG));
            let highlight = parse_highlight_flag(&mut args);
            follow_file(&file, highlight.as_deref())
        }
        "cat" => {
            let file = args.next().map(PathBuf::from).unwrap_or_else(|| PathBuf::from(DEFAULT_LOG));
            cat_file(&file)
        }
        "tcp" => {
            let port = args
                .next()
                .map(|s| s.parse::<u16>())
                .transpose()
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid port"))?
                .unwrap_or(DEFAULT_TCP_PORT);
            let host = parse_host_flag(&mut args).unwrap_or_else(|| "127.0.0.1".to_string());
            tcp_connect(&host, port)
        }
        "help" | "-h" | "--help" => {
            print_usage();
            Ok(())
        }
        other => {
            eprintln!("unknown command: {other}");
            print_usage();
            std::process::exit(1);
        }
    }
}

fn print_usage() {
    eprintln!(
        "Usage:
  aether-serial follow [FILE] [--highlight TEXT]
  aether-serial cat [FILE]
  aether-serial tcp [PORT] [--host ADDR]
  aether-serial help"
    );
}

fn parse_highlight_flag(args: &mut impl Iterator<Item = String>) -> Option<String> {
    let mut highlight = None;
    while let Some(arg) = args.next() {
        if arg == "--highlight" {
            highlight = args.next();
        }
    }
    highlight
}

fn parse_host_flag(args: &mut impl Iterator<Item = String>) -> Option<String> {
    let mut host = None;
    while let Some(arg) = args.next() {
        if arg == "--host" {
            host = args.next();
        }
    }
    host
}

fn cat_file(path: &Path) -> io::Result<()> {
    let file = File::open(path).map_err(|err| {
        io::Error::new(err.kind(), format!("failed to open {}: {err}", path.display()))
    })?;
    let mut stdout = io::stdout().lock();
    io::copy(&mut io::BufReader::new(file), &mut stdout)?;
    Ok(())
}

fn follow_file(path: &Path, highlight: Option<&str>) -> io::Result<()> {
    if !path.exists() {
        eprintln!("waiting for {} (run `make run` in another terminal)...", path.display());
        while !path.exists() {
            thread::sleep(Duration::from_millis(POLL_MS));
        }
    }

    let mut file = File::open(path)?;
    file.seek(SeekFrom::End(0))?;
    let mut reader = io::BufReader::new(file);
    let mut stdout = io::stdout().lock();

    loop {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(0) => {
                thread::sleep(Duration::from_millis(POLL_MS));
                continue;
            }
            Ok(_) => print_line(&mut stdout, &line, highlight)?,
            Err(err) if err.kind() == io::ErrorKind::Interrupted => continue,
            Err(err) => return Err(err),
        }
    }
}

fn print_line(
    stdout: &mut io::StdoutLock<'_>,
    line: &str,
    highlight: Option<&str>,
) -> io::Result<()> {
    if let Some(needle) = highlight {
        if line.contains(needle) {
            write!(stdout, "\x1b[1;32m{line}\x1b[0m")?;
            return Ok(());
        }
    }
    stdout.write_all(line.as_bytes())?;
    Ok(())
}

fn tcp_connect(host: &str, port: u16) -> io::Result<()> {
    let addr: SocketAddr = format!("{host}:{port}")
        .parse()
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidInput, err))?;

    eprintln!("connecting to {addr} (start QEMU with -serial tcp:localhost:{port},server,nowait)");
    let mut stream = TcpStream::connect(addr)?;
    stream.set_read_timeout(Some(Duration::from_millis(POLL_MS)))?;

    let mut stdout = io::stdout().lock();
    let mut buf = [0u8; 4096];

    loop {
        match stream.read(&mut buf) {
            Ok(0) => thread::sleep(Duration::from_millis(POLL_MS)),
            Ok(n) => {
                stdout.write_all(&buf[..n])?;
                stdout.flush()?;
            }
            Err(err) if err.kind() == io::ErrorKind::WouldBlock => {}
            Err(err) if err.kind() == io::ErrorKind::TimedOut => {}
            Err(err) => return Err(err),
        }
    }
}
