use std::sync::atomic::AtomicBool;
use std::time::Duration;

pub static SAVE_PATH: &str = "./log/file_paths.txt";
pub static LOG_PATH: &str = "log/file-logger.log";
pub static MINUTE_IN_SECONDS: u64 = 60;
pub static MINUTES: u64 = 5;
pub static SLEEP_DURATION: Duration = Duration::from_secs(MINUTE_IN_SECONDS * MINUTES);
pub static END: AtomicBool = AtomicBool::new(false);
