use std::fs::File;

use fs2::FileExt;

/// force a single running instance of the application via file locking
#[allow(dead_code)]
pub struct SingleInstance {
    lock_file: File,
}

impl SingleInstance {
    /// keep the instance in scope by holding it in a variable
    /// returns an error if there is already an instance running
    pub fn new(name: &str) -> Result<Self, ()> {
        let lock_file = File::create(format! {"{name}.lock"}).unwrap();
        lock_file.try_lock_exclusive().or(Err(()))?;
        Ok(SingleInstance { lock_file })
    }
}