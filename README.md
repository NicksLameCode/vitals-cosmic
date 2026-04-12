# vitals-cosmic

COSMIC desktop applet for system vitals — Gentoo / COSMIC port of the
[`vitals-rs`](https://github.com/NicksLameCode/vitals-rs) system monitor.

Thin D-Bus client of `com.corecoding.Vitals` (the `vitals-daemon` session
service shipped by `vitals-rs`), rendered in `libcosmic` / `iced`. Zero sensor
logic lives in this crate — it all comes from `vitals-core` via path dep.

## Architecture

```
+-------------------+     D-Bus     +--------------------+
|  vitals-daemon    |<------------->|  vitals-cosmic     |
|  (from vitals-rs) |    session    |  libcosmic applet  |
+-------------------+               +--------------------+
       |
       v
  /sys, /proc, hwmon, nvidia-smi, DRM
```

The daemon owns all hardware access and polls sensors on a timer. The applet
subscribes via iced's subscription system, displays a compact hot-sensor row
in the COSMIC panel, and opens a popup with category sections, star-to-pin,
and a simple hover history summary.

## Requirements (Gentoo)

- Rust 1.94+ (`dev-lang/rust` or `rust-bin`)
- `cosmic-base/cosmic-base` — provides `cosmic-panel` and other COSMIC pieces
- `vitals-rs` installed and `vitals-daemon` reachable on `$DBUS_SESSION_BUS_ADDRESS`
  — the daemon is responsible for actually reading sensors. It ships a D-Bus
  `.service` file so session auto-activation works once `vitals-rs` is installed.

## Build

```sh
# 1. Make sure the sibling vitals-rs checkout exists at ../vitals-rs
git clone https://github.com/NicksLameCode/vitals-rs.git ../vitals-rs

# 2. Build the daemon from vitals-rs (one-time, until packaged)
cargo build -p vitals-daemon --release --manifest-path ../vitals-rs/Cargo.toml
../vitals-rs/target/release/vitals-daemon &  # or install as a session service

# 3. Build the applet
cargo build --release
```

Verify the daemon is reachable:

```sh
dbus-send --session --print-reply \
  --dest=com.corecoding.Vitals /com/corecoding/Vitals \
  com.corecoding.Vitals.Sensors.GetReadings
```

## Install (manual)

```sh
cargo install --path . --root ~/.local
install -Dm644 data/com.corecoding.VitalsCosmic.desktop \
  ~/.local/share/applications/com.corecoding.VitalsCosmic.desktop
```

Then enable the applet from **COSMIC Settings → Desktop → Panel → Applets**.

## Install (Gentoo, personal overlay)

A skeleton ebuild lives at `gentoo/app-admin/vitals-cosmic/`. Drop it into a
personal overlay (e.g. `/var/db/repos/local/app-admin/vitals-cosmic/`), adjust
`SRC_URI` to match whatever release tarball you cut, and `emerge`:

```sh
sudo emerge --ask app-admin/vitals-cosmic
```

`vitals-rs` is listed as an `RDEPEND`; the daemon it provides must be
installed. You'll likely need to write an `app-admin/vitals-rs` ebuild too —
it isn't in `::gentoo`.

## Project layout

```
src/
  main.rs         # cosmic::applet::run
  app.rs          # cosmic::Application: state, update, subscription, popup action
  dbus.rs         # zbus proxy for com.corecoding.Vitals.Sensors + background poll task
  config.rs       # load/save AppConfig (re-uses vitals_core::config)
  format.rs       # thin wrapper over vitals_core::format::ValueFormatter
  model.rs        # Reading / Category / Snapshot types, parse helpers
  icons.rs        # embedded SVG icon handles
  view/
    panel.rs      # panel button + hot sensors chip row
    popup.rs      # dropdown: categories, star toggles, footer actions
    prefs.rs      # preferences sub-popup
    graph.rs      # hover history summary (sparkline + min/avg/max/last)
data/
  com.corecoding.VitalsCosmic.desktop
  icons/          # 12 symbolic SVGs (copied from vitals-rs/data/icons/gnome)
gentoo/
  app-admin/vitals-cosmic/
    vitals-cosmic-0.1.0.ebuild
    metadata.xml
```

## Feature status vs GNOME extension

- [x] Panel hot-sensors row (pinned keys from `config.hot_sensors`)
- [x] Category sections in popup with star-to-pin
- [x] Refresh / Launch monitor / Preferences footer buttons
- [x] Daemon-unreachable state with Retry button
- [x] Config round-trips through daemon `SetConfig` so vitals-rs and the GNOME
      extension see the same settings
- [x] Hover history: unicode sparkline + min/avg/max/last summary
- [x] Preferences: update interval, higher precision, alphabetize, hide zeros,
      temperature/memory/storage units, public IP, monitor command
- [ ] True Canvas-based history graph (the sparkline fallback is temporary —
      pin a libcosmic commit and implement `canvas::Program<Message>` properly)
- [ ] Drag-to-reorder hot sensors list in preferences
- [ ] Full i18n (gettext)

## Related projects

- [`vitals-rs`](https://github.com/NicksLameCode/vitals-rs) — the Rust Vitals
  rewrite for Fedora/GNOME, provides the daemon this applet talks to.
- [`Vitals`](https://github.com/corecoding/Vitals) — the original CoreCoding
  GNOME Shell extension.
