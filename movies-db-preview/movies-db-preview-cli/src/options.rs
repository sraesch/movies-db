use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use log::LevelFilter;

use movies_db_preview::Options as ServiceOptions;

#[derive(ValueEnum, Clone, Copy, Debug)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl From<LogLevel> for LevelFilter {
    fn from(value: LogLevel) -> Self {
        match value {
            LogLevel::Trace => LevelFilter::Trace,
            LogLevel::Debug => LevelFilter::Debug,
            LogLevel::Info => LevelFilter::Info,
            LogLevel::Warn => LevelFilter::Warn,
            LogLevel::Error => LevelFilter::Error,
        }
    }
}

/// CLI interface to test different occlusion culler algorithms.
#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Options {
    /// The log level
    #[arg(short, value_enum, long, default_value_t = LogLevel::Info)]
    pub log_level: LogLevel,

    /// The address to bind the http server to
    #[arg(short, value_enum, long, default_value = "0.0.0.0:3030")]
    pub address: String,

    /// The path to the root directory
    #[arg(short, long)]
    pub cache_dir: PathBuf,
}

impl From<Options> for ServiceOptions {
    fn from(options: Options) -> Self {
        ServiceOptions {
            cache_dir: options.cache_dir,
            http_address: options.address.parse().unwrap(),
        }
    }
}
