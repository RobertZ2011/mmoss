use std::ffi::{CString, c_char};

use log::Level;

struct Logger {
    callback: Option<unsafe extern "C" fn(level: u8, message: *const c_char)>,
}

impl log::Log for Logger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        if let Some(callback) = self.callback {
            let level = record.level() as u8;
            let message = CString::new(format!("{}", record.args())).unwrap();
            unsafe {
                callback(level, message.as_ptr());
            }
        }
    }

    fn flush(&self) {}
}

#[unsafe(no_mangle)]
pub extern "C" fn mmoss_init_log(
    level: u8,
    callback: Option<unsafe extern "C" fn(level: u8, message: *const c_char)>,
) {
    let level = match level {
        1 => Level::Error,
        2 => Level::Warn,
        3 => Level::Info,
        4 => Level::Debug,
        5 => Level::Trace,
        _ => {
            eprintln!("Invalid log level {}, defaulting to Info", level);
            Level::Info
        }
    };

    let logger = Logger { callback };
    log::set_max_level(level.to_level_filter());
    if let Err(e) = log::set_boxed_logger(Box::new(logger)) {
        eprintln!("Failed to set logger: {}", e);
    }
}
