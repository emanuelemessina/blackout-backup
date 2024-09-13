use std::sync::{Arc, RwLock};

use lazy_static::lazy_static;
use tokio::sync::futures::Notified;
use tokio::sync::Notify;

lazy_static! {
    pub static ref APP_STATE : Arc<ApplicationStateManager> = ApplicationStateManager::new();
}

#[derive(PartialEq, Clone, Copy)]
pub enum ApplicationState {
    Running,
    Quit
}

/// used to direct other threads flow without channels
/// threads that do not go to sleep can actively read the state and take action
/// threads that want to wait for changes without wasting cpu cycles can do so with the notify (wait channel)
/// a thread can also subscribe to the changes and be notified via a priority channel, either sync or async
pub struct ApplicationStateManager {
    state: RwLock<ApplicationState>,
    notify: Notify,
    subscribers: RwLock<Vec<tokio::sync::mpsc::Sender<ApplicationState>>>,
    subscribers_sync: RwLock<Vec<crossbeam::channel::Sender<ApplicationState>>>
}

impl ApplicationStateManager {
    fn new() -> Arc<Self> {
        Arc::new(
            ApplicationStateManager {
                state: RwLock::new(ApplicationState::Running),
                notify: Notify::new(),
                subscribers: RwLock::new(Vec::new()),
                subscribers_sync: RwLock::new(Vec::new())
            })
    }

    pub fn read(&self) -> ApplicationState {
        *(self.state.read().unwrap())
    }
    pub async fn change(&self, state: ApplicationState) {
        *self.state.write().unwrap() = state;
        self.notify.notify_waiters();
        let mut to_remove = Vec::new();
        for (idx, tx) in self.subscribers.read().unwrap().iter().enumerate() {
            if tx.send(state).await.is_err() {
                to_remove.push(idx);
            }
        }
        for idx in to_remove.iter() {
            self.subscribers.write().unwrap().remove(*idx);
        }
        to_remove.clear();
        for (idx, tx) in self.subscribers_sync.read().unwrap().iter().enumerate() {
            if tx.send(state).is_err() {
                to_remove.push(idx);
            }
        }
        for idx in to_remove.into_iter() {
            self.subscribers_sync.write().unwrap().remove(idx);
        }
    }
    #[allow(dead_code)]
    pub fn wait_for_change(&self) -> Notified<'_> {
        self.notify.notified()
    }

    pub fn subscribe(&self) -> tokio::sync::mpsc::Receiver<ApplicationState> {
        let (tx, rx) = tokio::sync::mpsc::channel(1);
        self.subscribers.write().unwrap().push(tx);
        rx
    }

    pub fn subscribe_sync(&self) -> crossbeam::channel::Receiver<ApplicationState> {
        let (tx, rx) = crossbeam::channel::bounded(1);
        self.subscribers_sync.write().unwrap().push(tx);
        rx
    }
}