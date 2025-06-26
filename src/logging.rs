use tracing::{Level, Subscriber};
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter, Registry,
};
use std::fs;
use std::path::PathBuf;
use anyhow::Result;
use crate::config::OmniConfig;

pub fn init_logging(config: &OmniConfig) -> Result<()> {
    let log_level = parse_log_level(&config.general.log_level);
    let log_dir = OmniConfig::data_dir()?.join("logs");
    fs::create_dir_all(&log_dir)?;
    
    let log_file = log_dir.join("omni.log");
    let file_appender = tracing_appender::rolling::daily(&log_dir, "omni.log");
    
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(format!("omni={}", log_level)));

    let file_layer = fmt::layer()
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_target(true)
        .with_span_events(FmtSpan::CLOSE)
        .with_writer(file_appender)
        .with_ansi(false);

    let stdout_layer = fmt::layer()
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .with_ansi(config.ui.use_colors)
        .compact();

    Registry::default()
        .with(env_filter)
        .with(file_layer)
        .with(stdout_layer)
        .init();

    Ok(())
}

fn parse_log_level(level_str: &str) -> &'static str {
    match level_str.to_lowercase().as_str() {
        "trace" => "trace",
        "debug" => "debug", 
        "info" => "info",
        "warn" | "warning" => "warn",
        "error" => "error",
        _ => "info",
    }
}

#[macro_export]
macro_rules! log_operation {
    ($level:ident, $operation:expr, $package:expr, $message:expr) => {
        tracing::$level!(
            operation = $operation,
            package = $package,
            "{}", $message
        );
    };
}

#[macro_export]
macro_rules! log_install {
    ($package:expr, $box_type:expr, $status:expr) => {
        tracing::info!(
            operation = "install",
            package = $package,
            box_type = $box_type,
            status = $status,
            "Package installation: {} via {} - {}", $package, $box_type, $status
        );
    };
}

#[macro_export]
macro_rules! log_error {
    ($operation:expr, $package:expr, $error:expr) => {
        tracing::error!(
            operation = $operation,
            package = $package,
            error = %$error,
            "Operation failed: {} for package {} - {}", $operation, $package, $error
        );
    };
}