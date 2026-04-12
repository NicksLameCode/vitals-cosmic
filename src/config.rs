//! Config persistence layer. We round-trip through the daemon's `SetConfig`
//! RPC whenever possible so both vitals-rs and vitals-cosmic see the same
//! `~/.config/vitals/config.toml`. If the daemon isn't reachable we fall back
//! to the local file.

use anyhow::Result;
use vitals_core::config::AppConfig;

/// Load the canonical config. Reads `~/.config/vitals/config.toml` directly;
/// the daemon writes to that file too, so local read is always fresh enough.
pub fn load() -> AppConfig {
    AppConfig::load()
}

/// Serialize a config for sending over D-Bus.
pub fn to_toml(config: &AppConfig) -> Result<String> {
    Ok(toml::to_string_pretty(config)?)
}
