use std::cmp::PartialEq;
use std::thread;
use std::time::Duration;

use mouse_position::mouse_position::Mouse;
use rdev::display_size;

use crate::{audio, tokio};
use crate::audio::{play_sound, SOUND_ARM, SOUND_CANCEL, SOUND_TRIGGER};
use crate::backup::backup_main;
use crate::logger::{error, info};
use crate::state::{APP_STATE, ApplicationState};
use crate::TOKIO;

#[allow(dead_code)]
#[derive(PartialEq)]
enum Position {
    TopLeft,
    TopRight,
    BottomRight,
    BottomLeft,
    Outside,
    Inside,
}
struct SafeArea {
    margin_left: u32,
    margin_right: u32,
    margin_top: u32,
    margin_bottom: u32
}

fn get_safe_area() -> SafeArea {
    let (width, height) = display_size().unwrap();
    let (width, height) = (width as u32, height as u32);
    let margin = width / 6;
    SafeArea {
        margin_top: margin,
        margin_left: margin,
        margin_bottom: height - margin,
        margin_right: width - margin
    }
}

/// this thread controls the backup triggering, it listens for mouse movements and plays audio
pub fn mouse_main() {
    let safe_area = get_safe_area();
    let mut position = Position::Outside;
    let mut armed = false;

    loop {
        if APP_STATE.read() == ApplicationState::Quit {
            break;
        }
        match Mouse::get_mouse_position() {
            Mouse::Position { x, y } => {
                let x = x as u32;
                let y = y as u32;

                if armed {
                    // second path
                    if path_completed(&mut position, &safe_area, x, y) {
                        // trigger
                        armed = false;
                        tokio!().spawn(play_sound(audio::PLAYER.clone(), SOUND_TRIGGER));
                        info!("mouse", "Backup triggered!");
                        // spawn backup thread
                        // the mouse thread is blocked until the backup finishes, then it's ready to fire again
                        thread::spawn(backup_main)
                            .join()
                            .inspect_err(|_e| { error!("mouse", "Backup thread panicked!"); })
                            .err();
                        info!("mouse", "Resumed probing mouse.");
                    } else if position == Position::Outside {
                        // fell out, disarm
                        armed = false;
                        tokio!().spawn(play_sound(audio::PLAYER.clone(), SOUND_CANCEL));
                        info!("mouse", "Backup disarmed.");
                    }
                } else {
                    // first path
                    if path_completed(&mut position, &safe_area, x, y) {
                        // arm
                        armed = true;
                        tokio!().spawn(play_sound(audio::PLAYER.clone(), SOUND_ARM));
                        info!("mouse", "Backup armed");
                    }
                }
            },
            Mouse::Error => {
                error!("mouse", "Error getting mouse position!");
                break;
            },
        }
        // this is a low level loop, we have no other choice than to sleep the thread
        // at least we can choose the update interval, and in this cas it can be very long
        thread::sleep(Duration::from_millis(200));
    }
}

/// returns true when the path is closed, then starts over again
fn path_completed(position: &mut Position, safe_area: &SafeArea, x: u32, y: u32) -> bool {
    if (x > safe_area.margin_left && x < safe_area.margin_right)
        && (y > safe_area.margin_top && y < safe_area.margin_bottom) {
        *position = Position::Outside;
        return false;
    }

    match position {
        Position::Outside | Position::TopRight => {
            if x < safe_area.margin_left && y < safe_area.margin_top {
                *position = Position::TopLeft;
            }
        },
        Position::TopLeft => {
            if x < safe_area.margin_left && y > safe_area.margin_bottom {
                *position = Position::BottomLeft;
            }
        },
        Position::BottomLeft => {
            if x > safe_area.margin_right && y > safe_area.margin_bottom {
                *position = Position::BottomRight;
            }
        },
        Position::BottomRight => {
            if x > safe_area.margin_right && y < safe_area.margin_top {
                *position = Position::TopRight;
                return true;
            }
        },
        Position::Inside => { *position = Position::Inside }
    }

    return false;
}



