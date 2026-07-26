//! Self-specified logger.
use std::io::Write;
use colored::Colorize;
use log::{Level, Log};

/// The main structure of this logger.
pub struct Logger;

impl Log for Logger {
    fn enabled(&self, _: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        match record.level() {
            Level::Error => eprintln!("{}: {}", "error".red().bold(), record.args()),
            Level::Warn => eprintln!("{}: {}", "warning".yellow().bold(), record.args()),
            Level::Info => println!("{}: {}", "info".bold(), record.args()),
            Level::Debug => println!("{}: {}", "debug".blue().bold(), record.args()),
            Level::Trace => println!("{}: {}", "trace".magenta().bold(), record.args()),
        }
    }

    fn flush(&self) {
        let _ = std::io::stdout().flush();
    }
}

/// Logger initializator.
pub fn init() {
    static LOGGER: Logger = Logger;

    // Set up logger
    log::set_logger(&LOGGER).expect("Failed to set up logger");

    // Set max level through debug assertions
    #[cfg(debug_assertions)]
    log::set_max_level(log::LevelFilter::Trace);

    #[cfg(not(debug_assertions))]
    log::set_max_level(log::LevelFilter::Info);
}
