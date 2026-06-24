use clap::{Parser, ValueEnum};
use std::fmt;
use std::path::PathBuf;

#[derive(ValueEnum, Debug, Clone)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogLevel::Error => write!(f, "error"),
            LogLevel::Warn => write!(f, "warn"),
            LogLevel::Info => write!(f, "info"),
            LogLevel::Debug => write!(f, "debug"),
            LogLevel::Trace => write!(f, "trace"),
        }
    }
}

#[derive(Parser, Debug)]
#[command(
    name = "smulx-dedup",
    version,
    about = "Find and manage duplicate images"
)]
pub struct Cli {
    /// Root directory to scan (can be repeated for multiple paths)
    #[arg(required = true)]
    pub paths: Vec<PathBuf>,

    /// Hamming distance threshold (0 = exact duplicates only, 10 = very similar)
    /// Recommended: 5
    #[arg(short, long, default_value_t = 5)]
    pub threshold: u32,

    /// Use system trash instead of permanent deletion
    #[arg(long, default_value_t = true)]
    pub use_trash: bool,

    /// Export cluster list to JSON before opening TUI
    #[arg(long)]
    pub export_json: Option<PathBuf>,

    /// Log level (error, warn, info, debug, trace)
    #[arg(long, default_value = "warn", env = "SMULX_LOG")]
    pub log_level: LogLevel,
}
