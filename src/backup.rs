use std::error::Error;
use std::fs::{copy, create_dir_all, File, metadata, OpenOptions};
use std::io::{BufRead, BufReader};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, mpsc};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::thread;
use std::time::{Duration, Instant};

use chksum_hash_md5 as md5;
use chrono::Local;
use glob::glob;
use sysinfo::Disks;
use walkdir::WalkDir;

use crate::{audio, tokio};
use crate::audio::{heartbeat, play_sound, SOUND_ERROR, SOUND_SUCCESS};
use crate::echo::echo_main;
use crate::logger::{error, info};
use crate::TOKIO;

pub fn backup_main() {
    // we can ignore the errors because ui is non critical for the backup operation
    let (tx, rx) = mpsc::channel();

    let echo = thread::spawn(move || {
        echo_main(rx);
    });

    tx.send("░       ░░░  ░░░░░░░░░      ░░░░      ░░░  ░░░░  ░░░      ░░░  ░░░░  ░░        ░".to_string()).unwrap();
    tx.send("▒  ▒▒▒▒  ▒▒  ▒▒▒▒▒▒▒▒  ▒▒▒▒  ▒▒  ▒▒▒▒  ▒▒  ▒▒▒  ▒▒▒  ▒▒▒▒  ▒▒  ▒▒▒▒  ▒▒▒▒▒  ▒▒▒▒".to_string()).unwrap();
    tx.send("▓       ▓▓▓  ▓▓▓▓▓▓▓▓  ▓▓▓▓  ▓▓  ▓▓▓▓▓▓▓▓     ▓▓▓▓▓  ▓▓▓▓  ▓▓  ▓▓▓▓  ▓▓▓▓▓  ▓▓▓▓".to_string()).unwrap();
    tx.send("█  ████  ██  ████████        ██  ████  ██  ███  ███  ████  ██  ████  █████  ████".to_string()).unwrap();
    tx.send("█       ███        ██  ████  ███      ███  ████  ███      ████      ██████  ████\n".to_string()).unwrap();

    let msg = "Parsing sources...".to_string();
    info!("backup", msg.clone());
    tx.send(msg).unwrap();

    let heartbeat_stop = Arc::new(AtomicBool::new(false));
    tokio!().spawn(heartbeat(audio::PLAYER.clone(), Duration::from_secs(1), heartbeat_stop.clone()));

    let mut error_msg = String::default();
    match parse_sources() {
        Err(e) => { error_msg = format!("Error parsing resources! {}", e); },
        Ok((paths, size, checksum)) => {
            if paths.len() > 0 {
                let suitable_mounts = find_suitable_mounts(size);

                if suitable_mounts.len() == 0 {
                    error_msg = format!("No removable drives with enough free space found! Required space: {}", size.human_readable());
                } else {
                    let (dest, available_space) = &suitable_mounts[0];

                    let msg = format!("Found {} as suitable destination drive. Available space: {}. Required space {}. Started copying files...",
                                      dest.to_string_lossy(), available_space.human_readable(), size.human_readable());
                    info!("backup", msg.clone());
                    tx.send(msg).unwrap();

                    match copy_files(&paths, dest, checksum, size, tx.clone()) {
                        Err(e) => {
                            error_msg = format!("Error copying files! {}", e);
                        },
                        Ok(_) => {
                            tokio!().spawn(play_sound(audio::PLAYER.clone(), SOUND_SUCCESS));
                        }
                    }
                }
            }
        }
    }

    if !error_msg.is_empty() {
        error!("backup", error_msg.clone());
        tx.send(error_msg).unwrap();
        tokio!().spawn(play_sound(audio::PLAYER.clone(), SOUND_ERROR));
    }

    heartbeat_stop.store(true, Ordering::Release);

    let msg = "Backup finished.".to_string();
    info!("backup", msg.clone());
    tx.send(msg).unwrap();

    thread::sleep(Duration::from_secs(5)); // give user some time to view the console

    drop(tx); // close echo

    echo.join()
        .inspect_err(|_e| { error!("backup", "Echo thread panicked!"); })
        .err();
}

/// returns (on success) a vector of (pathbuf, size) of files, the total size, and the sources file checksum
fn parse_sources() -> Result<(Vec<(PathBuf, u64)>, u64, String), Box<dyn Error>> {
    let mut parsed = Vec::new();
    let mut tot_size: u64 = 0;

    let file = File::open("sources.txt")?;

    let reader = BufReader::new(file);

    // we parse the file and calculate the has at the same time
    let mut checksum = md5::default();

    for line in reader.lines() {
        let line = line.unwrap();

        checksum.update(&line);

        let path_pattern = Path::new(&line);

        if path_pattern.to_string_lossy().contains('*') || path_pattern.to_string_lossy().contains('?') {
            let pattern = line.trim();
            let paths = glob(pattern)?;

            for entry in paths {
                match entry {
                    Ok(entry) => {
                        let size = metadata(&entry).unwrap().len();
                        tot_size += size;
                        parsed.push((entry, size));
                    },
                    Err(e) => {
                        return Err(e.into());
                    }
                }
            }
        } else {
            let path = Path::new(&line);
            if path.exists() {
                if path.is_dir() {
                    for entry in WalkDir::new(path).into_iter().filter_map(Result::ok) {
                        let size = metadata(entry.path()).unwrap().len();
                        parsed.push((PathBuf::from(entry.path()), size));
                        tot_size += size;
                    }
                } else {
                    let size = metadata(PathBuf::from(path)).unwrap().len();
                    parsed.push((PathBuf::from(path), size));
                    tot_size += size;
                }
            } else {
                return Err(format!("Path does not exist: {}", path.display()).into());
            }
        }
    }

    return Ok((parsed, tot_size, checksum.digest().to_hex_lowercase()));
}

fn find_suitable_mounts(size: u64) -> Vec<(PathBuf, u64)> {
    let disks = Disks::new_with_refreshed_list();
    let suitable = disks.list().into_iter().filter(|disk| disk.is_removable() && size < disk.available_space());
    let mut mount_points = Vec::new();
    for disk in suitable {
        mount_points.push((disk.mount_point().to_owned(), disk.available_space()));
    }
    mount_points
}

trait HumanReadable {
    fn human_readable(&self) -> String;
}

impl HumanReadable for u64 {
    fn human_readable(&self) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;
        const TB: u64 = GB * 1024;

        if *self >= TB {
            format!("{:.2} TB", *self as f64 / TB as f64)
        } else if *self >= GB {
            format!("{:.2} GB", *self as f64 / GB as f64)
        } else if *self >= MB {
            format!("{:.2} MB", *self as f64 / MB as f64)
        } else if *self >= KB {
            format!("{:.2} KB", *self as f64 / KB as f64)
        } else {
            format!("{} bytes", *self)
        }
    }
}

impl HumanReadable for Duration {
    fn human_readable(&self) -> String {
        let hours = self.as_secs() / 3600;
        let minutes = (self.as_secs() % 3600) / 60;
        let seconds = self.as_secs() % 60;
        let milliseconds = self.subsec_millis();

        format!(
            "{:02}h{:02}m:{:02}s::{:03}",
            hours, minutes, seconds, milliseconds
        )
    }
}

fn copy_files(files: &Vec<(PathBuf, u64)>, dest: &PathBuf, checksum: String, total_size: u64, ui: Sender<String>) -> Result<(), Box<dyn Error>> {
    let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
    let backup_root = dest.join(PathBuf::from("BlackoutBackup"));
    let backup_path = backup_root.join(format!("{}", timestamp));

    create_dir_all(&backup_path)?;

    let mut backup_log = OpenOptions::new().write(true).create(true).open(backup_root.join(format!("{}.log", timestamp)))?;

    writeln!(backup_log, "Sources checksum: {}\n", checksum).unwrap();

    let mut written_size: u64 = 0;

    let start = Instant::now();

    for (file, size) in files {
        // change drive in path
        let path_no_drive = file.strip_prefix(file.ancestors().last().unwrap()).unwrap();
        let dest_path = backup_path.join(path_no_drive);

        let msg = format!("Copying {} ...", file.to_string_lossy());
        info!("backup", msg.clone());
        ui.send(msg).unwrap();

        if let Some(parent) = dest_path.parent() {
            // don't block the entire backup if one file fails
            create_dir_all(parent).inspect_err(|e| {
                let msg = format!("{e}");
                error!("backup", msg.clone());
                ui.send(msg).unwrap();
            }).err();
            copy(file, dest_path).inspect_err(|e| {
                let msg = format!("{e}");
                error!("backup", msg.clone());
                ui.send(msg).unwrap();
            }).err();

            written_size += size;

            writeln!(backup_log, "{:>10} | {}", size.human_readable(), path_no_drive.to_string_lossy()).unwrap();
        }
    }

    let duration = start.elapsed();

    let msg = format!("Written: {} / {}", written_size.human_readable(), total_size.human_readable());
    writeln!(backup_log, "\n{}", msg.clone()).unwrap();
    info!("backup", msg.clone());
    ui.send(msg).unwrap();

    let msg = format!("Time elapsed: {}", duration.human_readable());
    writeln!(backup_log, "\n{}", msg.clone()).unwrap();
    info!("backup", msg.clone());
    ui.send(msg).unwrap();

    Ok(())
}