use std::fs;
use tracing::Level;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Layer;

fn logs_dir() -> std::path::PathBuf {
    let base = dirs::data_dir()
        .or_else(dirs::config_dir)
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    base.join("WhisperKey").join("logs")
}

pub fn init(level: &str) {
    let lvl: Level = level.parse().unwrap_or(Level::INFO);

    fs::create_dir_all(logs_dir()).ok();

    let file_appender = RollingFileAppender::new(Rotation::DAILY, logs_dir(), "app");

    let file_layer = tracing_subscriber::fmt::layer()
        .json()
        .with_writer(file_appender)
        .with_filter(tracing_subscriber::filter::LevelFilter::from_level(lvl));

    let console_layer = tracing_subscriber::fmt::layer()
        .with_filter(tracing_subscriber::filter::LevelFilter::from_level(lvl));

    tracing_subscriber::registry()
        .with(file_layer)
        .with(console_layer)
        .init();
}
