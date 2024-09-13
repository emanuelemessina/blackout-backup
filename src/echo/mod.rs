use std::error::Error;
use std::io::{BufWriter, Write};
use std::process::{Command, Stdio};
use std::sync::mpsc::Receiver;

use crate::logger::{error, info};
use crate::state::{APP_STATE, ApplicationState};
use crate::TOKIO;
use crate::tokio;

pub struct Echo {
    child: Option<std::process::Child>,
    stdin_writer: Option<BufWriter<std::process::ChildStdin>>,
}

impl Echo {
    pub(crate) fn new() -> Self {
        Self {
            child: None,
            stdin_writer: None,
        }
    }

    pub(crate) fn spawn(&mut self) -> Result<(), Box<dyn Error>> {
        let mut child = Command::new("echo.exe")
            .stdin(Stdio::piped())
            .spawn()?;

        let child_stdin = child.stdin.take().ok_or("Failed to get stdin!")?;
        self.child = Some(child);
        let stdin_writer = BufWriter::new(child_stdin);
        self.stdin_writer = Some(stdin_writer);
        Ok(())
    }

    pub(crate) fn write(&mut self, message: String) -> Result<(), Box<dyn Error>> {
        if let Some(writer) = &mut self.stdin_writer {
            writeln!(writer, "{}", message)?;
            writer.flush()?;
        }
        Ok(())
    }

    pub(crate) fn check_alive(&mut self) -> bool {
        if let Some(child) = self.child.as_mut() {
            child.try_wait().ok().flatten().is_none()
        } else { false }
    }

    pub fn kill(&mut self) -> Result<(), Box<dyn Error>> {
        if let Some(child) = self.child.as_mut() {
            child.kill()?;
        }
        Ok(())
    }
}

/// will respawn the echo process if is closed externally, to quit just drop the sender
pub fn echo_main(rx: Receiver<String>) {
    let mut echo = Echo::new();
    while let Ok(message) = rx.recv() {
        if APP_STATE.read() == ApplicationState::Quit {
            break;
        }
        if !echo.check_alive() {
            match echo.spawn() {
                Err(e) => {
                    error!("echo", format!("Fatal error! {:?}", e));
                    tokio!().block_on(APP_STATE.change(ApplicationState::Quit));
                    break;
                },
                _ => {}
            }
        }
        match echo.write(message) {
            Err(e) => {
                error!("echo", format!("Fatal error! {:?}", e));
                tokio!().block_on(APP_STATE.change(ApplicationState::Quit));
                break;
            }
            _ => {}
        }
    }
    drop(rx);
    info!("echo", "Echo quitting...");
    match echo.kill() {
        Err(e) => {
            error!("echo", format!("Fatal error! {:?}", e));
            tokio!().block_on(APP_STATE.change(ApplicationState::Quit));
        },
        Ok(_) => {
            info!("echo", "Echo killed.");
        }
    }
}