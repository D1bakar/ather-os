//! Minimal Aether OS shell — `echo` and `help` commands.

#![cfg_attr(not(feature = "host"), no_std)]
#![cfg_attr(not(feature = "host"), no_main)]

use aether_rt::{exit, print, println};

const PROMPT: &str = "aether> ";

#[cfg(feature = "host")]
fn main() -> ! {
    shell_main()
}

#[cfg(not(feature = "host"))]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    shell_main()
}

#[cfg(not(feature = "host"))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}

fn shell_main() -> ! {
    println("Aether shell (M6 minimal)");
    println("Type 'help' for commands.");

    loop {
        print(PROMPT);
        let line = read_line();
        if line.is_empty() {
            continue;
        }
        dispatch(&line);
    }
}

fn dispatch(line: &str) {
    let mut parts = line.split_whitespace();
    let cmd = parts.next().unwrap_or("");

    match cmd {
        "help" => {
            println("Commands:");
            println("  help       — show this message");
            println("  echo TEXT  — print TEXT");
            println("  exit       — terminate shell");
        }
        "echo" => {
            let rest = line[cmd.len()..].trim_start();
            println(rest);
        }
        "exit" => exit(0),
        "" => {}
        other => {
            print("unknown command: ");
            println(other);
            println("Type 'help' for available commands.");
        }
    }
}

#[cfg(feature = "host")]
fn read_line() -> String {
    use std::io::{self, BufRead};
    let stdin = io::stdin();
    let mut line = String::new();
    match stdin.lock().read_line(&mut line) {
        Ok(0) => exit(0),
        Ok(_) => {
            if line.ends_with('\n') {
                line.pop();
                if line.ends_with('\r') {
                    line.pop();
                }
            }
            line
        }
        Err(_) => String::new(),
    }
}

#[cfg(not(feature = "host"))]
fn read_line() -> &'static str {
    // Bare metal: no stdin driver yet — demo with a fixed script.
    println("(no stdin driver — demo commands only)");
    println("help");
    "help"
}
