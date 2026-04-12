//! Domain types shared across modules. The D-Bus wire format uses plain strings
//! for category and format; we parse them into the [`vitals_core`] enums so the
//! rest of the applet can reuse `ValueFormatter` unchanged.

use std::collections::BTreeMap;

use vitals_core::sensors::SensorFormat;

/// A single sensor reading as received from the daemon.
#[derive(Debug, Clone)]
pub struct Reading {
    pub key: String,
    pub label: String,
    pub category: Category,
    pub kind: ReadingKind,
}

#[derive(Debug, Clone)]
pub enum ReadingKind {
    Numeric { value: f64, format: SensorFormat },
    Text(String),
}

/// Ordered sensor categories, matching the GNOME extension's grouping.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Category {
    Temperature,
    Voltage,
    Fan,
    Memory,
    Processor,
    System,
    Network,
    Storage,
    Battery,
    Gpu(u8),
}

impl Category {
    /// Parse the category string produced by `vitals_core::SensorCategory::Display`.
    /// Examples: "temperature", "gpu#0".
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "temperature" => Some(Self::Temperature),
            "voltage" => Some(Self::Voltage),
            "fan" => Some(Self::Fan),
            "memory" => Some(Self::Memory),
            "processor" => Some(Self::Processor),
            "system" => Some(Self::System),
            "network" => Some(Self::Network),
            "storage" => Some(Self::Storage),
            "battery" => Some(Self::Battery),
            other => {
                let n = other.strip_prefix("gpu#")?.parse().ok()?;
                Some(Self::Gpu(n))
            }
        }
    }

    /// Display name for the popup section header.
    pub fn title(&self) -> String {
        match self {
            Self::Temperature => "Temperature".into(),
            Self::Voltage => "Voltage".into(),
            Self::Fan => "Fans".into(),
            Self::Memory => "Memory".into(),
            Self::Processor => "Processor".into(),
            Self::System => "System".into(),
            Self::Network => "Network".into(),
            Self::Storage => "Storage".into(),
            Self::Battery => "Battery".into(),
            Self::Gpu(n) => format!("GPU {n}"),
        }
    }

    /// Icon key (matches SVG filenames bundled in `data/icons/`).
    pub fn icon_name(&self) -> &'static str {
        match self {
            Self::Temperature => "temperature",
            Self::Voltage => "voltage",
            Self::Fan => "fan",
            Self::Memory => "memory",
            Self::Processor => "processor",
            Self::System => "system",
            Self::Network => "network",
            Self::Storage => "storage",
            Self::Battery => "battery",
            Self::Gpu(_) => "gpu",
        }
    }

    /// Stable display order for the popup.
    pub fn display_order() -> &'static [Category] {
        &[
            Self::Processor,
            Self::Memory,
            Self::System,
            Self::Network,
            Self::Storage,
            Self::Temperature,
            Self::Fan,
            Self::Voltage,
            Self::Battery,
            // GPUs intentionally omitted — inserted dynamically in popup view.
        ]
    }
}

/// Parse the SensorFormat string produced by `SensorFormat::as_str()`.
pub fn parse_format(s: &str) -> Option<SensorFormat> {
    Some(match s {
        "percent" => SensorFormat::Percent,
        "temp" => SensorFormat::Temp,
        "fan" => SensorFormat::Fan,
        "in" => SensorFormat::Voltage,
        "hertz" => SensorFormat::Hertz,
        "memory" => SensorFormat::Memory,
        "storage" => SensorFormat::Storage,
        "speed" => SensorFormat::Speed,
        "uptime" => SensorFormat::Uptime,
        "runtime" => SensorFormat::Runtime,
        "watt" => SensorFormat::Watt,
        "watt-gpu" => SensorFormat::WattGpu,
        "watt-hour" => SensorFormat::WattHour,
        "milliamp" => SensorFormat::Milliamp,
        "milliamp-hour" => SensorFormat::MilliampHour,
        "load" => SensorFormat::Load,
        "pcie" => SensorFormat::Pcie,
        "string" => SensorFormat::StringVal,
        _ => return None,
    })
}

/// One poll's worth of data; sent from the D-Bus task to the UI.
#[derive(Debug, Clone, Default)]
pub struct Snapshot {
    pub readings: BTreeMap<String, Reading>,
}

impl Snapshot {
    pub fn grouped(&self) -> BTreeMap<Category, Vec<&Reading>> {
        let mut out: BTreeMap<Category, Vec<&Reading>> = BTreeMap::new();
        for r in self.readings.values() {
            out.entry(r.category).or_default().push(r);
        }
        for group in out.values_mut() {
            group.sort_by(|a, b| a.label.cmp(&b.label));
        }
        out
    }
}

/// Current connectivity state to the vitals-daemon.
#[derive(Debug, Clone)]
pub enum DaemonState {
    Connecting,
    Connected,
    Unreachable(String),
}
