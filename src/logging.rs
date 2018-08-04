//! Contains functions to help with logging.

use fern;
use chrono;
use log;

use std::io;

pub fn setup_logger() -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Info)
        .chain(io::stdout())
        .chain(fern::log_file("installer.log")?)
        .apply()?;
    Ok(())
}
