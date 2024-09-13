use crossbeam::channel::bounded;
use crossbeam::select;
use tray_item::{IconSource, TrayItem};

use crate::{TOKIO, tokio};
use crate::logger::info;
use crate::state::{APP_STATE, ApplicationState};

enum Action {
    Quit,
}

pub fn tray_main() {
    // define tray
    let mut tray = TrayItem::new(
        "Blackout",
        IconSource::Resource("icon"),
    ).unwrap();

    // event channel
    let (tx, rx) = bounded(1);

    // event senders
    let quit_tx = tx.clone();
    tray.add_menu_item("Quit", move || {
        quit_tx.send(Action::Quit).unwrap();
    }).unwrap();

    // state

    let state_rx = APP_STATE.subscribe_sync();

    // event handlers

    loop {
        select! {
            recv(rx) -> action => { // tray events
                 match action {
                    Ok(Action::Quit) => {
                        info!("tray", "Quitted from tray.");
                        tokio!().block_on(APP_STATE.change(ApplicationState::Quit));
                        break;
                    },
                    _ => {}
                }
            },
            recv(state_rx) -> state => {
                 match state {
                    Ok(ApplicationState::Quit) => { break; },
                    _ => {}
                };
            }
        }
    }
}