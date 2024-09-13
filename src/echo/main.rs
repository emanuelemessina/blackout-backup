#![
    windows_subsystem = "windows"
]

use std::io;
use std::io::{BufRead, stdout, Write};

use winapi::um::consoleapi::AllocConsole;
use winapi::um::wincon::FreeConsole;

fn main() {
    unsafe {
        FreeConsole();

        if AllocConsole() == 0 {
            eprintln!("Failed to allocate console");
            return;
        }
    }

    let stdin = io::stdin();
    let stdin_lock = stdin.lock();
    let lines = stdin_lock.lines();
    for line in lines {
        match line {
            Ok(text) => {
                println!("{}", text);
                stdout().flush().unwrap();
            }
            Err(e) => {
                eprintln!("Error reading line: {}", e);
                break;
            }
        }
    }
}