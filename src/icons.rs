//! Embedded SVG icons (copied from `vitals-rs/data/icons/gnome`).
//!
//! We embed the SVG bytes at compile time so the applet works no matter where
//! it's installed (no icon-theme lookup path to worry about).

use cosmic::widget::icon::{self, Handle};

const TEMPERATURE: &[u8] = include_bytes!("../data/icons/temperature-symbolic.svg");
const VOLTAGE: &[u8] = include_bytes!("../data/icons/voltage-symbolic.svg");
const FAN: &[u8] = include_bytes!("../data/icons/fan-symbolic.svg");
const MEMORY: &[u8] = include_bytes!("../data/icons/memory-symbolic.svg");
const CPU: &[u8] = include_bytes!("../data/icons/cpu-symbolic.svg");
const SYSTEM: &[u8] = include_bytes!("../data/icons/system-symbolic.svg");
const NETWORK: &[u8] = include_bytes!("../data/icons/network-symbolic.svg");
const NETWORK_DOWN: &[u8] = include_bytes!("../data/icons/network-download-symbolic.svg");
const NETWORK_UP: &[u8] = include_bytes!("../data/icons/network-upload-symbolic.svg");
const STORAGE: &[u8] = include_bytes!("../data/icons/storage-symbolic.svg");
const BATTERY: &[u8] = include_bytes!("../data/icons/battery-symbolic.svg");
const GPU: &[u8] = include_bytes!("../data/icons/gpu-symbolic.svg");

fn handle(bytes: &'static [u8]) -> Handle {
    icon::from_svg_bytes(bytes).symbolic(true)
}

pub fn for_category(name: &str) -> Handle {
    match name {
        "temperature" => handle(TEMPERATURE),
        "voltage" => handle(VOLTAGE),
        "fan" => handle(FAN),
        "memory" => handle(MEMORY),
        "processor" => handle(CPU),
        "system" => handle(SYSTEM),
        "network" => handle(NETWORK),
        "network-down" => handle(NETWORK_DOWN),
        "network-up" => handle(NETWORK_UP),
        "storage" => handle(STORAGE),
        "battery" => handle(BATTERY),
        "gpu" => handle(GPU),
        _ => handle(SYSTEM),
    }
}
