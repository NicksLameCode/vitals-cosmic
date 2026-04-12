<div align="center">

<br>

<img src="https://raw.githubusercontent.com/NicksLameCode/vitals-cosmic/main/data/icons/cpu-symbolic.svg" width="80" alt="vitals-cosmic">

# vitals-cosmic

### Your system's vital signs, in your COSMIC panel.

A native [libcosmic](https://github.com/pop-os/libcosmic) applet port of
[vitals-rs](https://github.com/NicksLameCode/vitals-rs) for the
[COSMIC desktop](https://system76.com/cosmic). Thin D-Bus client of the
`vitals-daemon` shipped by vitals-rs -- the same sensor backend powers the
GNOME Shell extension and the GTK4 desktop app on Fedora, and now this applet
on Gentoo COSMIC.

<br>

[![License](https://img.shields.io/badge/License-BSD_3--Clause-blue?style=for-the-badge)](LICENSE)
&nbsp;
[![Rust](https://img.shields.io/badge/Rust-2021_Edition-f74c00?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
&nbsp;
[![libcosmic](https://img.shields.io/badge/libcosmic-iced-7c3aed?style=for-the-badge&logo=rust&logoColor=white)](https://github.com/pop-os/libcosmic)
&nbsp;
[![Platform](https://img.shields.io/badge/Platform-Gentoo_%2F_COSMIC-54487a?style=for-the-badge&logo=gentoo&logoColor=white)]()

<br>

<!-- Replace with real screenshot when available -->
<!--
<img src="data/screenshots/vitals-cosmic-panel.png" alt="vitals-cosmic panel" width="720">
<br><br>
-->

[Features](#features) &#8226;
[Architecture](#architecture) &#8226;
[Installation](#installation) &#8226;
[Usage](#usage) &#8226;
[Configuration](#configuration) &#8226;
[Development](#development)

<br>

</div>

---

<br>

## Features

<table>
<tr>
<td width="50%" valign="top">

**Native COSMIC Integration**
- Built on `libcosmic` and `iced`, no GTK runtime needed
- Follows the same layout recipe as `cosmic-applet-time` --
  `button::custom` inside `widget::autosize::autosize`, symbolic icons
  tinted with the panel theme's foreground color
- Popup opens via `surface::action::app_popup` with click-positioned
  anchoring so it drops below the applet button correctly
- Preferences sub-popup rendered in the same surface

</td>
<td width="50%" valign="top">

**Feature Parity with the GNOME Extension**
- Panel chip row: an icon + value for every pinned hot sensor
- Full category dropdown (temperature / voltage / fan / memory / CPU /
  system / network / storage / battery / GPU)
- Star-to-pin any sensor onto the panel, persisted across restarts
- Hover a sensor to see its history summary inline
- Refresh / launch-monitor / preferences footer actions
- Graceful "daemon unreachable" state with a Retry button

</td>
</tr>
<tr>
<td width="50%" valign="top">

**Shared Backend with vitals-rs**
- Path-deps on `../vitals-rs/crates/vitals-core` -- no code duplication
- Config round-trips through the daemon's `SetConfig` RPC, so vitals-rs,
  the GNOME extension, and this applet all see the same
  `~/.config/vitals/config.toml`
- Same 10 sensor categories, same formatters, same history store
- Switching desktops doesn't mean losing your pinned sensors

</td>
<td width="50%" valign="top">

**Packaging Ready**
- Skeleton Gentoo ebuild at `gentoo/app-admin/vitals-cosmic/`
- Embedded symbolic SVGs (`include_bytes!`) -- no icon-theme lookup at runtime
- D-Bus session auto-activation via the daemon's existing `.service` file
- Clean release build (no warnings), `lto = "thin"` + `strip = true`
- Generation-counter guard for the hover-history race condition
  inherited from the GNOME extension

</td>
</tr>
</table>

<br>

---

<br>

## Architecture

```
                    +------------------+
                    |   vitals-core    |    Pure Rust library
                    |                  |    Shared with vitals-rs
                    |  Sensors, Config |    via cargo path dep
                    |  Format, History |
                    +--------+---------+
                             |
                             v
                    +------------------+
                    |  vitals-daemon   |    D-Bus service
                    |                  |    com.corecoding.Vitals
                    |  Polls sensors   |    session bus
                    |  on a timer      |
                    +--------+---------+
                             |
                             | D-Bus (GetReadings, GetTimeSeries,
                             |        SetConfig, ReadingsChanged)
                             |
                    +--------v---------+
                    |  vitals-cosmic   |    libcosmic applet
                    |                  |    (this crate)
                    |  Panel button +  |    Loaded by cosmic-panel
                    |  category popup  |    as a child process
                    +------------------+
```

| Component | Description |
|:----------|:------------|
| **vitals-core** | Sensor discovery and polling, hwmon parsing, TOML config, value formatting, time-series history. Lives in the sibling [vitals-rs](https://github.com/NicksLameCode/vitals-rs) repo; pulled in as a Cargo path dependency. |
| **vitals-daemon** | Headless D-Bus service on the session bus. Also from vitals-rs. Auto-activated via `com.corecoding.Vitals.service` on first RPC. |
| **vitals-cosmic** | The applet itself -- D-Bus client, iced `Application`, panel button + popup + preferences views. |

The applet holds zero sensor-reading code. Everything flows through the
daemon's four RPCs (`GetReadings`, `GetTextReadings`, `GetTimeSeries`,
`GetConfig` / `SetConfig`). This matches the GNOME Shell extension's
architecture, so all three clients stay in sync.

<br>

---

<br>

## Installation

### Dependencies

- Rust **1.94+** (for the `edition = "2024"` `vitals-core` dependency)
- `cosmic-base` -- provides `cosmic-panel` and the rest of the COSMIC
  session. On Gentoo this comes from the [pop-os-overlay](https://github.com/pop-os/pop-overlay)
  or [cosmic-overlay](https://github.com/fsvm88/cosmic-overlay).
- A running `vitals-daemon` from
  [vitals-rs](https://github.com/NicksLameCode/vitals-rs). Installed system-wide
  or to `~/.local/bin` with a matching D-Bus service file.

<details>
<summary><strong>Gentoo</strong></summary>

```bash
# System dependencies
sudo emerge --ask cosmic-base/cosmic-base dev-lang/rust virtual/pkgconfig

# Clone vitals-rs (provides the daemon + vitals-core)
git clone https://github.com/NicksLameCode/vitals-rs.git ../vitals-rs
cargo build -p vitals-daemon --release --manifest-path ../vitals-rs/Cargo.toml
```
</details>

<details>
<summary><strong>Other distros with COSMIC</strong></summary>

Any distro shipping `libcosmic`, `cosmic-panel`, and a modern Rust toolchain
should work. Fedora/Pop!_OS/Arch packages exist in various stages of
readiness -- check your distro's docs for COSMIC setup, then follow the
manual install below.
</details>

### Build from source

```bash
git clone https://github.com/NicksLameCode/vitals-cosmic.git
cd vitals-cosmic
cargo build --release
```

The release binary lands at `target/release/vitals-cosmic` (LTO + stripped,
~20 MiB).

### Manual install (user-local)

```bash
# 1. Install the daemon from vitals-rs
install -Dm755 ../vitals-rs/target/release/vitals-daemon ~/.local/bin/vitals-daemon
install -Dm644 /dev/stdin ~/.local/share/dbus-1/services/com.corecoding.Vitals.service <<EOF
[D-BUS Service]
Name=com.corecoding.Vitals
Exec=$HOME/.local/bin/vitals-daemon
EOF

# 2. Install vitals-cosmic
install -Dm755 target/release/vitals-cosmic ~/.local/bin/vitals-cosmic
install -Dm644 data/com.corecoding.VitalsCosmic.desktop \
    ~/.local/share/applications/com.corecoding.VitalsCosmic.desktop
install -Dm644 data/icons/cpu-symbolic.svg \
    ~/.local/share/icons/hicolor/scalable/apps/com.corecoding.VitalsCosmic-symbolic.svg

# 3. Patch the desktop file's Exec= to an absolute path
#    (cosmic-panel's PATH does NOT include ~/.local/bin)
sed -i "s|^Exec=vitals-cosmic$|Exec=$HOME/.local/bin/vitals-cosmic|" \
    ~/.local/share/applications/com.corecoding.VitalsCosmic.desktop
```

### Enable in the COSMIC panel

Edit `~/.config/cosmic/com.system76.CosmicPanel.Panel/v1/plugins_wings`
and add `"com.corecoding.VitalsCosmic"` to one of the wing lists, e.g.:

```ron
Some(([
    "com.system76.CosmicPanelWorkspacesButton",
    "com.system76.CosmicPanelAppButton",
], [
    "com.system76.CosmicAppletStatusArea",
    "com.corecoding.VitalsCosmic",
    "com.system76.CosmicAppletAudio",
    // ... rest of the right wing
]))
```

Reload the panel:

```bash
kill -HUP $(pgrep ^cosmic-panel$)
```

(`cosmic-session` respawns it automatically.)

### Gentoo ebuild (personal overlay)

A skeleton ebuild lives at `gentoo/app-admin/vitals-cosmic/`. Drop it into
a personal overlay and adjust `SRC_URI` for whichever tag you cut. It
`RDEPEND`s on `app-admin/vitals-rs` (also not in `::gentoo` -- write a
companion ebuild).

```bash
sudo cp -r gentoo/app-admin/vitals-cosmic /var/db/repos/local/app-admin/
sudo ebuild /var/db/repos/local/app-admin/vitals-cosmic/vitals-cosmic-0.1.0.ebuild manifest
sudo emerge --ask app-admin/vitals-cosmic
```

<br>

---

<br>

## Usage

Once installed and registered with cosmic-panel, the applet shows as a row
of `(icon value)` chips -- one chip per pinned "hot sensor". Click the
applet to open the full category popup, where you can:

- Expand any category to see its sensors
- Click the star next to a sensor to pin or unpin it from the panel
- Hover a sensor row to see a compact history summary underneath
- Hit **Refresh** to force an immediate daemon poll
- Hit **Open monitor** to launch your configured system monitor
  (`btop` by default on this port)
- Hit **Preferences** to open the settings sub-popup

### Verify the daemon is reachable

```bash
dbus-send --session --print-reply \
    --dest=com.corecoding.Vitals /com/corecoding/Vitals \
    com.corecoding.Vitals.Sensors.GetReadings
```

You should see a dict of `(label, value, category, format)` tuples. If the
daemon isn't running, D-Bus auto-activation via the `.service` file will
start it on demand.

### Standalone dev loop

```bash
COSMIC_PANEL_APPLET=1 ./target/release/vitals-cosmic
```

Opens a floating popup window instead of embedding in the panel. Useful
for iterating on the popup view without reloading cosmic-panel every time.

<br>

---

<br>

## Feature status vs the GNOME extension

| | vitals-cosmic | GNOME extension (`vitals-rs/extension/`) |
|:--|:--:|:--:|
| Panel hot-sensors row | ✅ | ✅ |
| Category popup with star-to-pin | ✅ | ✅ |
| Preferences dialog | ✅ | ✅ |
| Refresh / launch-monitor / preferences footer | ✅ | ✅ |
| Daemon-unreachable state with Retry | ✅ | ✅ |
| Config shared across clients via daemon `SetConfig` | ✅ | ✅ |
| Hover history | ✅ sparkline + min/avg/max/last | ✅ Cairo line graph |
| i18n (gettext) | 🚧 | ✅ (20 langs) |
| Drag-to-reorder hot sensors in prefs | 🚧 | ✅ |
| Canvas `canvas::Program`-based line graph | 🚧 follow-up | n/a |

The sparkline hover summary is a placeholder -- replacing it with a proper
iced `canvas::Program` needs a pinned libcosmic commit so the generic
`Theme`/`Renderer` bounds stay stable between builds.

<br>

---

<br>

## Configuration

Config lives at `~/.config/vitals/config.toml`, shared with vitals-rs. The
applet reads it at startup and pushes every preferences change back through
`com.corecoding.Vitals.Sensors.SetConfig(toml)` so the daemon writes the
canonical copy.

<details>
<summary><strong>Relevant sections</strong></summary>

```toml
[general]
update_time = 5              # Seconds between daemon polls (1-60)
use_higher_precision = false # Extra decimal digit in values
alphabetize = true           # Sort sensors alphabetically within each section
hide_zeros = false           # Hide rows whose value is exactly 0
monitor_cmd = "btop"         # Launched by the "Open monitor" footer button

[temperature]
unit = 0                     # 0 = Celsius, 1 = Fahrenheit

[memory]
measurement = 1              # 0 = binary (GiB), 1 = decimal (GB)

[storage]
measurement = 1              # 0 = binary (GiB), 1 = decimal (GB)

[network]
include_public_ip = true     # Include public IP in the network section

# Keys of sensors pinned onto the panel chip row. Set via star toggle in
# the popup; max 4 visible on the panel at once.
hot_sensors = [
    "_memory_usage_",
    "_processor_total_",
]
```

See the [vitals-rs config reference](https://github.com/NicksLameCode/vitals-rs#configuration)
for the complete list of fields.
</details>

<br>

---

<br>

## Development

### Quick start

```bash
git clone https://github.com/NicksLameCode/vitals-rs.git ../vitals-rs
git clone https://github.com/NicksLameCode/vitals-cosmic.git
cd vitals-cosmic

cargo build                                    # Debug build
cargo build --release                          # LTO + stripped
cargo run -- COSMIC_PANEL_APPLET=1              # Dev loop (floating popup)
```

### Linting

```bash
cargo fmt
cargo clippy -- -D warnings
```

### Project structure

```
vitals-cosmic/
  Cargo.toml                      # Path-dep on ../vitals-rs/crates/vitals-core
  src/
    main.rs                       # cosmic::applet::run::<Vitals>
    app.rs                        # cosmic::Application: state, update, subscription,
                                  # popup action factory
    dbus.rs                       # zbus #[proxy] for com.corecoding.Vitals.Sensors
                                  # + Tokio background poll task
    config.rs                     # Thin wrapper around vitals_core::config::AppConfig
    format.rs                     # Wrapper around vitals_core::format::ValueFormatter
    model.rs                      # Reading / Category / Snapshot domain types
    icons.rs                      # Embedded SVG handles
    view/
      panel.rs                    # Panel chip row (button_custom + autosize)
      popup.rs                    # Category dropdown with star toggles
      graph.rs                    # Hover history summary (sparkline + stats)
      prefs.rs                    # Preferences sub-popup
  data/
    com.corecoding.VitalsCosmic.desktop
    icons/                        # 12 symbolic SVGs (copied from vitals-rs)
  gentoo/
    app-admin/vitals-cosmic/
      vitals-cosmic-0.1.0.ebuild
      metadata.xml
```

### Key design notes

- **Panel layout** follows the `cosmic-applet-time` recipe: a `button::custom`
  wrapped in `widget::autosize::autosize(btn, AUTOSIZE_MAIN_ID)` so
  cosmic-panel sizes the applet slot to fit the chip row instead of
  clipping to a fixed icon width.
- **Symbolic icons** need explicit theming via
  `Svg::Custom(|theme| ... theme.cosmic().background.on ...)` to inherit
  the panel's foreground color, otherwise they render with the SVG's
  baked-in fill.
- **Popup** uses `surface::action::app_popup::<Vitals>(settings, view)` --
  the more recent libcosmic popup API -- instead of the older
  `platform_specific::shell::wayland::commands::popup::get_popup` path
  used by the stock applets. Either works; this is the one documented in
  the current `examples/applet/src/window.rs`.
- **Hover history race**: a `hover_generation` counter on `Vitals` is
  incremented on every `HoverEnter` and checked when `HistoryUpdated`
  arrives, so a slow `GetTimeSeries` response for a previously hovered
  sensor never paints over the current one. Ported from
  [vitals-rs commit `06d529b`](https://github.com/NicksLameCode/vitals-rs/commit/06d529b).

<br>

---

<br>

## Credits

vitals-cosmic is built on top of:

- **[vitals-rs](https://github.com/NicksLameCode/vitals-rs)** -- the Rust
  sensor library, daemon, and companion GTK4 app / GNOME Shell extension.
- **[Vitals](https://github.com/corecoding/Vitals)** by
  [corecoding](https://github.com/corecoding) -- the original GNOME Shell
  extension whose feature set and UX are the north star for both ports.
- **[libcosmic](https://github.com/pop-os/libcosmic)** by System76 -- the
  Rust/iced widget toolkit powering the COSMIC desktop.
- **[cosmic-applet-time](https://github.com/pop-os/cosmic-applets/tree/master/cosmic-applet-time)** --
  the reference implementation for a text-in-panel COSMIC applet that this
  port's panel layout is modeled after.

## License

Licensed under the [BSD-3-Clause License](LICENSE), matching vitals-rs.

<br>
