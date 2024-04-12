use colored::Colorize;
use log::{Level, LevelFilter, Metadata, Record, SetLoggerError};
use supports_color::Stream;

struct SimpleLogger {
    colored: bool,
}

impl SimpleLogger {
    pub fn new() -> Self {
        Self {
            colored: supports_color::on(Stream::Stdout).is_some()
        }
    }
}

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        let level_str = {
            let mut lowercase_str = record.level().as_str().to_lowercase();
            if self.colored {
                match record.level() {
                    Level::Info => lowercase_str = lowercase_str.green().bold().to_string(),
                    Level::Error => lowercase_str = lowercase_str.red().bold().to_string(),
                    Level::Warn => lowercase_str = lowercase_str.yellow().bold().to_string(),
                    _ => {}
                };
            }
            lowercase_str
        };

        if self.enabled(record.metadata()) {
            println!("{}: {}", level_str, record.args());
        }
    }

    fn flush(&self) {}
}

pub fn init() -> Result<(), SetLoggerError> {
    log::set_boxed_logger(Box::from(SimpleLogger::new()))
        .map(|()| log::set_max_level(LevelFilter::Info))
}