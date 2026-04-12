//! D-Bus client for `com.corecoding.Vitals.Sensors` (the vitals-daemon session
//! bus service).  Mirrors the interface defined in
//! `vitals-rs/crates/vitals-daemon/src/dbus.rs`.

use std::collections::HashMap;
use std::time::Duration;

use anyhow::{Context, Result};
use futures_util::StreamExt;
use tokio::sync::mpsc;
use zbus::Connection;

use crate::model::{parse_format, Category, DaemonState, Reading, ReadingKind, Snapshot};

/// Raw tuple shape as sent over D-Bus: (label, value, category, format).
type NumericTuple = (String, f64, String, String);
/// (label, text, category, format).
type TextTuple = (String, String, String, String);

#[zbus::proxy(
    interface = "com.corecoding.Vitals.Sensors",
    default_service = "com.corecoding.Vitals",
    default_path = "/com/corecoding/Vitals"
)]
pub trait VitalsSensors {
    fn get_readings(&self) -> zbus::Result<HashMap<String, NumericTuple>>;
    fn get_text_readings(&self) -> zbus::Result<HashMap<String, TextTuple>>;
    fn get_time_series(&self, key: &str) -> zbus::Result<Vec<(f64, f64)>>;
    fn get_config(&self) -> zbus::Result<String>;
    fn set_config(&self, toml_str: &str) -> zbus::Result<bool>;

    #[zbus(signal)]
    fn readings_changed(&self) -> zbus::Result<()>;
}

/// Commands the UI can send to the background D-Bus task.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum DbusCmd {
    /// Force an immediate poll (user clicked "Refresh").
    RefreshNow,
    /// Fetch time-series history for a sensor key (hover graph).
    FetchHistory(String),
    /// Push a new config to the daemon and ask it to persist.
    SaveConfig(String),
    /// Change how often the background task polls.
    SetInterval(Duration),
    /// Shut the task down.
    Shutdown,
}

/// Events flowing from the background task up to the UI.
#[derive(Debug, Clone)]
pub enum DbusEvent {
    State(DaemonState),
    Snapshot(Box<Snapshot>),
    History {
        key: String,
        points: Vec<(f64, f64)>,
    },
    SaveResult(bool),
}

/// Spawn the daemon polling task. Returns the command sender.
/// The caller provides a sender that delivers [`DbusEvent`]s to the UI thread
/// via iced's subscription channel.
pub fn spawn(
    ui_tx: mpsc::Sender<DbusEvent>,
    initial_interval: Duration,
) -> mpsc::Sender<DbusCmd> {
    let (cmd_tx, cmd_rx) = mpsc::channel::<DbusCmd>(32);
    tokio::spawn(run(ui_tx, cmd_rx, initial_interval));
    cmd_tx
}

async fn run(
    ui_tx: mpsc::Sender<DbusEvent>,
    mut cmd_rx: mpsc::Receiver<DbusCmd>,
    initial_interval: Duration,
) {
    let mut interval = initial_interval;
    let _ = ui_tx.send(DbusEvent::State(DaemonState::Connecting)).await;

    loop {
        match Connection::session().await {
            Ok(conn) => match VitalsSensorsProxy::new(&conn).await {
                Ok(proxy) => {
                    let _ = ui_tx.send(DbusEvent::State(DaemonState::Connected)).await;
                    if let Err(e) = poll_loop(&proxy, &ui_tx, &mut cmd_rx, &mut interval).await {
                        log::warn!("poll_loop exited: {e:#}");
                        let _ = ui_tx
                            .send(DbusEvent::State(DaemonState::Unreachable(e.to_string())))
                            .await;
                    } else {
                        return; // Shutdown requested.
                    }
                }
                Err(e) => {
                    let _ = ui_tx
                        .send(DbusEvent::State(DaemonState::Unreachable(format!(
                            "proxy: {e}"
                        ))))
                        .await;
                }
            },
            Err(e) => {
                let _ = ui_tx
                    .send(DbusEvent::State(DaemonState::Unreachable(format!(
                        "session bus: {e}"
                    ))))
                    .await;
            }
        }

        // Retry with backoff; also drain any queued commands so shutdown still works.
        tokio::select! {
            _ = tokio::time::sleep(Duration::from_secs(3)) => {}
            Some(cmd) = cmd_rx.recv() => {
                if matches!(cmd, DbusCmd::Shutdown) {
                    return;
                }
            }
        }
    }
}

async fn poll_loop(
    proxy: &VitalsSensorsProxy<'_>,
    ui_tx: &mpsc::Sender<DbusEvent>,
    cmd_rx: &mut mpsc::Receiver<DbusCmd>,
    interval: &mut Duration,
) -> Result<()> {
    // Emit immediately on connect.
    send_snapshot(proxy, ui_tx).await?;

    let mut ticker = tokio::time::interval(*interval);
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    // Prime so the first tick fires after the interval elapses, not immediately.
    ticker.tick().await;

    // Best-effort signal subscription; if it fails we just rely on the ticker.
    let mut signal_stream = proxy.receive_readings_changed().await.ok();

    loop {
        tokio::select! {
            _ = ticker.tick() => {
                send_snapshot(proxy, ui_tx).await?;
            }
            Some(cmd) = cmd_rx.recv() => {
                match cmd {
                    DbusCmd::RefreshNow => {
                        send_snapshot(proxy, ui_tx).await?;
                    }
                    DbusCmd::FetchHistory(key) => {
                        let points = proxy
                            .get_time_series(&key)
                            .await
                            .context("GetTimeSeries")?;
                        let _ = ui_tx.send(DbusEvent::History { key, points }).await;
                    }
                    DbusCmd::SaveConfig(toml) => {
                        let ok = proxy.set_config(&toml).await.unwrap_or(false);
                        let _ = ui_tx.send(DbusEvent::SaveResult(ok)).await;
                    }
                    DbusCmd::SetInterval(new_iv) => {
                        *interval = new_iv;
                        ticker = tokio::time::interval(*interval);
                        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
                        ticker.tick().await;
                    }
                    DbusCmd::Shutdown => {
                        return Ok(());
                    }
                }
            }
            Some(_) = async {
                match signal_stream.as_mut() {
                    Some(s) => s.next().await,
                    None => None,
                }
            } => {
                send_snapshot(proxy, ui_tx).await?;
            }
        }
    }
}

async fn send_snapshot(
    proxy: &VitalsSensorsProxy<'_>,
    ui_tx: &mpsc::Sender<DbusEvent>,
) -> Result<()> {
    let numeric = proxy.get_readings().await.context("GetReadings")?;
    let text = proxy.get_text_readings().await.context("GetTextReadings")?;

    let mut snap = Snapshot::default();

    for (key, (label, value, cat, fmt)) in numeric {
        let Some(category) = Category::parse(&cat) else {
            continue;
        };
        let Some(format) = parse_format(&fmt) else {
            continue;
        };
        snap.readings.insert(
            key.clone(),
            Reading {
                key,
                label,
                category,
                kind: ReadingKind::Numeric { value, format },
            },
        );
    }
    for (key, (label, text, cat, _fmt)) in text {
        let Some(category) = Category::parse(&cat) else {
            continue;
        };
        snap.readings.insert(
            key.clone(),
            Reading {
                key,
                label,
                category,
                kind: ReadingKind::Text(text),
            },
        );
    }

    ui_tx
        .send(DbusEvent::Snapshot(Box::new(snap)))
        .await
        .map_err(|_| anyhow::anyhow!("ui receiver closed"))?;
    Ok(())
}
