use anyhow::Result;
use serde::Deserialize;
use std::{fmt, io::Write};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use time_tz::OffsetDateTimeExt;
use tracing::Level;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{
    EnvFilter,
    fmt::{format::Writer, time::FormatTime},
    layer::SubscriberExt,
    util::SubscriberInitExt,
};

use crate::general;

pub struct TzTimer;

impl FormatTime for TzTimer {
    fn format_time(&self, w: &mut Writer<'_>) -> fmt::Result {
        write!(
            w,
            "{}",
            OffsetDateTime::now_utc()
                .to_timezone(general::timezone())
                .format(&Rfc3339)
                .unwrap()
        )
    }
}

#[derive(Debug, Deserialize)]
pub struct LoggingConfig {
    pub level: LogLevel,
    pub writer: LogWriter,
    pub directory: String,
    pub file_name_prefix: String,
}

#[derive(Debug, Deserialize)]
pub enum LogLevel {
    #[serde(rename = "trace")]
    Trace,
    #[serde(rename = "debug")]
    Debug,
    #[serde(rename = "info")]
    Info,
    #[serde(rename = "warn")]
    Warn,
    #[serde(rename = "error")]
    Error,
}

#[derive(Debug, Deserialize)]
pub enum LogWriter {
    #[serde(rename = "file")]
    File,
    #[serde(rename = "stdout")]
    Stdout,
}

impl LogLevel {
    pub fn to_tracing_level(&self) -> Level {
        match self {
            LogLevel::Trace => Level::TRACE,
            LogLevel::Debug => Level::DEBUG,
            LogLevel::Info => Level::INFO,
            LogLevel::Warn => Level::WARN,
            LogLevel::Error => Level::ERROR,
        }
    }
}

pub fn init(config: &LoggingConfig) -> Result<WorkerGuard> {
    tracing_appender::rolling::set_tz(general::timezone())?;
    let (writer, ansi): (Box<dyn Write + Send + 'static>, bool) = match config.writer {
        LogWriter::File => (
            Box::new(tracing_appender::rolling::daily(
                config.directory.as_str(),
                config.file_name_prefix.as_str(),
            )),
            false,
        ),

        LogWriter::Stdout => (Box::new(std::io::stdout()), true),
    };
    let (non_blocking, worker_guard) = tracing_appender::non_blocking(writer);

    let filter =
        EnvFilter::from_default_env().add_directive(config.level.to_tracing_level().into());

    let layer = tracing_subscriber::fmt::layer()
        .with_ansi(ansi)
        .with_timer(TzTimer)
        .with_writer(non_blocking);
    tracing_subscriber::registry()
        .with(filter)
        .with(layer)
        .init();
    Ok(worker_guard)
}
