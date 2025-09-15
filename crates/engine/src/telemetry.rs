use serde::Serialize;
use std::fs::File;
use std::io::{self, Write};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::config::TelemetryConfig;

/// Minimal telemetry emitter that writes newline-delimited JSON events.
pub struct Telemetry {
    writer: Mutex<Box<dyn Write + Send>>,
}

impl Telemetry {
    /// Creates a telemetry instance from configuration. Returns `Ok(None)` when telemetry is disabled.
    pub fn from_config(cfg: &TelemetryConfig) -> io::Result<Option<Self>> {
        if !cfg.enabled {
            return Ok(None);
        }
        let writer: Box<dyn Write + Send> = if let Some(path) = &cfg.file {
            Box::new(File::create(path)?)
        } else {
            Box::new(io::stdout())
        };
        Ok(Some(Self {
            writer: Mutex::new(writer),
        }))
    }

    fn emit<T: Serialize>(&self, event: &T) {
        if let Ok(mut w) = self.writer.lock() {
            if serde_json::to_writer(&mut *w, event).is_ok() {
                let _ = w.write_all(b"\n");
            }
        }
    }

    /// Emits a `run_started` event with the current timestamp (ms since UNIX epoch).
    pub fn run_started(&self) {
        #[derive(Serialize)]
        struct RunStarted {
            event: &'static str,
            timestamp_ms: u128,
        }
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);
        self.emit(&RunStarted {
            event: "run_started",
            timestamp_ms: ts,
        });
    }

    /// Emits a `finding` event for the given rule and location.
    pub fn finding(&self, file: &str, line: usize, rule: &str) {
        #[derive(Serialize)]
        struct Finding<'a> {
            event: &'static str,
            file: &'a str,
            line: usize,
            rule: &'a str,
        }
        self.emit(&Finding {
            event: "finding",
            file,
            line,
            rule,
        });
    }

    /// Emits a `run_finished` event with summary statistics.
    pub fn run_finished(&self, findings: usize, duration_ms: u128) {
        #[derive(Serialize)]
        struct RunFinished {
            event: &'static str,
            findings: usize,
            duration_ms: u128,
        }
        self.emit(&RunFinished {
            event: "run_finished",
            findings,
            duration_ms,
        });
    }
}
