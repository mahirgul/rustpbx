use std::fs::OpenOptions;
use std::io::Write;
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

static SIP_LOG_FILE: OnceLock<Mutex<std::fs::File>> = OnceLock::new();
static AUTH_LOG_FILE: OnceLock<Mutex<std::fs::File>> = OnceLock::new();
static SYS_LOG_FILE: OnceLock<Mutex<std::fs::File>> = OnceLock::new();

pub fn init_separate_loggers() -> std::io::Result<()> {
    std::fs::create_dir_all("logs")?;

    let sip_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("logs/sip_messages.log")?;

    let auth_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("logs/auth_audit.log")?;

    let sys_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("logs/system.log")?;

    let _ = SIP_LOG_FILE.set(Mutex::new(sip_file));
    let _ = AUTH_LOG_FILE.set(Mutex::new(auth_file));
    let _ = SYS_LOG_FILE.set(Mutex::new(sys_file));

    Ok(())
}

fn get_timestamp() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("[{}.{:03}]", now.as_secs(), now.subsec_millis())
}

pub fn log_sip_message(direction: &str, target: &str, content: &str) {
    if let Some(mutex) = SIP_LOG_FILE.get() {
        if let Ok(mut file) = mutex.lock() {
            let entry = format!(
                "{} {} {}\n{}\n----------------------------------------\n",
                get_timestamp(),
                direction,
                target,
                content
            );
            let _ = file.write_all(entry.as_bytes());
        }
    }
}

pub fn log_auth_audit(event: &str) {
    if let Some(mutex) = AUTH_LOG_FILE.get() {
        if let Ok(mut file) = mutex.lock() {
            let entry = format!("{} [AUTH] {}\n", get_timestamp(), event);
            let _ = file.write_all(entry.as_bytes());
        }
    }
}

pub fn log_system_event(event: &str) {
    if let Some(mutex) = SYS_LOG_FILE.get() {
        if let Ok(mut file) = mutex.lock() {
            let entry = format!("{} [SYSTEM] {}\n", get_timestamp(), event);
            let _ = file.write_all(entry.as_bytes());
        }
    }
}
