#![cfg_attr(
    not(debug_assertions),
    windows_subsystem = "windows"
)] // remove console in release

use std::process::ExitCode;
use std::sync::OnceLock;
use std::thread;
use std::time::Duration;

use lazy_static::lazy_static;
use native_dialog::{MessageDialog, MessageType};
use sysinfo::System;
use tokio::time;

use single_instance::SingleInstance;

use crate::logger::info;
use crate::mouse::mouse_main;
use crate::state::{APP_STATE, ApplicationState};
use crate::tray::tray_main;

mod logger;
mod single_instance;
mod tray;
mod mouse;
mod audio;
mod backup;
mod echo;
mod state;

lazy_static! {
    pub static ref TOKIO : OnceLock<tokio::runtime::Handle> = OnceLock::new();
}
/// gets the current handle of tokio, must be called after the main set the handle
#[macro_export]
macro_rules! tokio {
    () => {
        TOKIO.get().unwrap()
    };
}

#[tokio::main]
async fn main() -> ExitCode {

    // prologue

    let _single_instance = SingleInstance::new("blackout");
    if _single_instance.is_err() {
        MessageDialog::new()
            .set_type(MessageType::Warning)
            .set_title("Blackout")
            .set_text("Blackout is already running.")
            .show_alert()
            .unwrap();
        return ExitCode::FAILURE
    }

    logger::spawn!("blackout");

    info!("main", "Process started.");

    // tokio

    TOKIO.set(tokio::runtime::Handle::current()).unwrap();

    // state

    let mut state_rx = APP_STATE.subscribe();

    // tray

    let tray = thread::spawn(tray_main);

    // mouse

    let mouse = thread::spawn(mouse_main);

    // cpu

    let two_mins = Duration::from_secs(120);
    let mut system = System::new_all();
    let pid = (std::process::id() as usize).into();
    let num_cores = system.cpus().len();

    // main thread

    loop {
        tokio::select! {
            Some(state) = state_rx.recv() => { // state machine
                match state {
                    ApplicationState::Quit => { break; },
                    _ => {}
                };
            },
            _ = time::sleep(two_mins) => { // wake every 2 minutes
                // cpu usage log
                system.refresh_cpu_usage();
                let mut cpu_usage = system.process(pid).unwrap().cpu_usage();
                cpu_usage /= num_cores as f32;
                info!("main", format!("CPU usage: {:.2}%", cpu_usage));
            }
        }
    }

    // join other threads
    tray.join().unwrap();
    mouse.join().unwrap();

    // epilogue

    info!("main", "Process terminated.");

    logger::flush!();

    return ExitCode::SUCCESS;
}

