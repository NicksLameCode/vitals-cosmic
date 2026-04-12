//! Thin wrapper around `vitals_core::format::ValueFormatter` so the rest of the
//! applet doesn't need to know about the config ownership dance.

use vitals_core::config::AppConfig;
use vitals_core::format::ValueFormatter;

use crate::model::{Reading, ReadingKind};

pub fn format_reading(config: &AppConfig, reading: &Reading) -> String {
    let f = ValueFormatter::new(config);
    match &reading.kind {
        ReadingKind::Numeric { value, format } => f.format(*value, *format),
        ReadingKind::Text(s) => s.clone(),
    }
}
