use std::fs::{create_dir_all, OpenOptions};
use std::io::{stderr, Write};
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

static LOG_FILE: OnceLock<Mutex<std::fs::File>> = OnceLock::new();

#[macro_export]
macro_rules! app_log {
    ($($arg:tt)*) => {{
        $crate::logger::log_line(format!($($arg)*));
    }};
}

fn resolve_log_path() -> Option<PathBuf> {
    let home = std::env::var_os("HOME")?;
    let mut path = PathBuf::from(home);
    path.push("Library");
    path.push("Logs");
    path.push("Voxlore");
    Some(path)
}

pub fn init_file_logger() {
    #[cfg(target_os = "macos")]
    {
        let Some(log_dir) = resolve_log_path() else {
            return;
        };

        if create_dir_all(&log_dir).is_err() {
            return;
        }

        let log_path = log_dir.join("run.log");
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path);

        if let Ok(file) = file {
            let _ = LOG_FILE.set(Mutex::new(file));
            log_line("[logger] file logger ready: ~/Library/Logs/Voxlore/run.log".to_string());
        }
    }
}

pub fn log_line(message: String) {
    let _ = writeln!(stderr(), "{message}");

    #[cfg(target_os = "macos")]
    if let Some(lock) = LOG_FILE.get() {
        if let Ok(mut file) = lock.lock() {
            let _ = writeln!(file, "{message}");
        }
    }
}

