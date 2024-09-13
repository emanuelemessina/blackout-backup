use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use lazy_static::lazy_static;
use soloud::{audio, AudioExt, LoadExt, Soloud};

pub const SOUND_ARM: &[u8] = include_bytes!("../res/arm.mp3");
pub const SOUND_CANCEL: &[u8] = include_bytes!("../res/cancel.mp3");
pub const SOUND_TRIGGER: &[u8] = include_bytes!("../res/trigger.mp3");
pub const SOUND_ERROR: &[u8] = include_bytes!("../res/error.mp3");
pub const SOUND_HEARTBEAT: &[u8] = include_bytes!("../res/heartbeat.mp3");
pub const SOUND_SUCCESS: &[u8] = include_bytes!("../res/success.mp3");

pub type Player = Arc<Soloud>;

lazy_static! {
    pub static ref PLAYER : Player = Arc::new(Soloud::default().unwrap());
}

pub async fn play_sound(sl: Player, data: &[u8]) {
    let mut wav = audio::Wav::default();
    wav.load_mem(data).unwrap();
    sl.play(&wav); // there is memory corruption is the wav is shared on a higher level and the thread quits (?!)
    while sl.voice_count() > 0 {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
}

pub async fn heartbeat(sl: Player, tick: Duration, terminate: Arc<AtomicBool>) {
    loop {
        play_sound(sl.clone(), SOUND_HEARTBEAT).await;
        tokio::time::sleep(tick).await; // precision is not critical, low overhead with respect to interval.tick
        if terminate.load(Ordering::Acquire) {
            break;
        }
    }
}
