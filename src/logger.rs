#![allow(unused_imports)]
#![allow(unused_macros)]

use std::{fmt, io, thread};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::JoinHandle;

use chrono::Local;

// DEFINITIONS

#[allow(dead_code)]
#[derive(PartialEq)]
pub enum LogLevel {
    Info,
    Error
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let level_str = match self {
            LogLevel::Info => "Info",
            LogLevel::Error => "Error",
        };
        write!(f, "{:<5}", level_str)
    }
}

/// Log message
struct Log {
    to: Option<String>,
    level: Option<LogLevel>,
    from: String,
    body: String,
}

/// Thread safe async logger. It runs its own thread to write logs to files without blocking the caller thread.
pub struct Logger {
    thread: Option<JoinHandle<()>>,
    tx: Option<Sender<Option<Log>>>,
}

// ROUTINE

/// Listens for messages in the channel and writes them to their respective files
fn logger_main(default_file: String, rx: Receiver<Option<Log>>) {
    let mut log_files: HashMap<String, File> = HashMap::new();
    get_log_file(&mut log_files, default_file.as_str()).unwrap(); // insert the default file
    loop {
        let message = rx.recv().unwrap();
        if message.is_some() {
            let log = message.unwrap();
            let mut file = get_log_file(&mut log_files, log.to.unwrap_or(default_file.clone()).as_str()).unwrap();
            let timestamp = Local::now().format("%Y-%m-%dT%H:%M:%S%.3f").to_string();
            let mut line = format!("{timestamp} | ");
            if log.level.is_some() {
                line.push_str(format!("{} | ", log.level.unwrap()).as_str());
            }
            line.push_str(format!("{:<15} : {}", log.from, log.body).as_str());
            writeln!(file, "{line}").unwrap();
        } else { break; /* termination condition, the logger will log all remaining messages before exiting */ }
    }
}


/// Retrieve a log file a log file handle by name, or create one if not present
fn get_log_file(log_files: &mut HashMap<String, File>, name: &str) -> io::Result<File> {
    if let Some(file) = log_files.get(name) {
        file.try_clone()
    } else {
        let file = OpenOptions::new().append(true).create(true).open(format!("{}.log", name))?;
        log_files.insert(name.to_string(), file.try_clone()?);
        Ok(file)
    }
}

// IMPL

impl Logger {
    /// logs to the default file
    pub fn log(&self, level: LogLevel, from: String, body: String) {
        let log = Log {
            to: None,
            level: Some(level),
            from,
            body
        };
        if let Some(tx) = &self.tx {
            tx.send(Some(log)).unwrap()
        }
    }
    /// logs to an external file
    #[allow(dead_code)]
    pub fn out(&self, to: String, from: String, body: String) {
        let log = Log {
            to: Some(to),
            level: None,
            from,
            body
        };
        if let Some(tx) = &self.tx {
            tx.send(Some(log)).unwrap()
        }
    }

    /// spawns the logger thread, specify the default file to use for the main log
    pub fn start(&mut self, default_file: String) {
        let (tx, rx) = channel();
        let thread = thread::spawn(move || { logger_main(default_file, rx) });
        self.tx = Some(tx);
        self.thread = Some(thread);
    }

    /// explicit stop method because for statics drop is not called
    pub fn stop(&mut self) {
        if let Some(tx) = &self.tx {
            tx.send(None).unwrap();
            self.tx = None;
        }
        if let Some(thread) = self.thread.take() {
            thread.join().unwrap();
            self.thread = None;
        }
    }
}

// GLOBAL

pub static mut LOGGER: Logger = Logger { thread: None, tx: None };

// MACROS

// start is a reserved attribute it seems...
macro_rules! spawn {
    ($default_file:expr) => {
        unsafe{
            crate::logger::LOGGER.start($default_file.to_string());
        }
    }
}

pub(crate) use spawn;

/// Signals the logger to write all pending logs and joins. Call it at the end of main
macro_rules! flush {
    () => {
        unsafe{
           crate::logger::LOGGER.stop();
        }
    }
}

pub(crate) use flush;

macro_rules! info {
    ($from:expr, $body:expr) => {
        unsafe {
            crate::logger::LOGGER.log(crate::logger::LogLevel::Info, $from.to_string(), $body.to_string());
        }
    };
}

pub(crate) use info;

macro_rules! error {
    ($from:expr, $body:expr) => {
        unsafe {
            crate::logger::LOGGER.log(crate::logger::LogLevel::Error, $from.to_string(), $body.to_string());
        }
    };
}

pub(crate) use error;

macro_rules! out {
    ($from:expr, $body:expr) => {
        unsafe {
            crate::logger::LOGGER.out($to.to_string(), $from.to_string(), $body.to_string());
        }
    };
}

pub(crate) use out;