//! Top-level [`cosmic::Application`] for the vitals-cosmic applet.
//!
//! The applet talks to the `com.corecoding.Vitals` D-Bus service provided by
//! `vitals-daemon`, displays a compact hot-sensor row in the COSMIC panel,
//! and opens a popup with all sensors when clicked.

use std::time::Duration;

use cosmic::app::{Core, Task};
use cosmic::iced::{stream, window, Rectangle, Subscription, Vector};
use cosmic::surface::action::{app_popup, destroy_popup};
use cosmic::{Application, Element};
use tokio::sync::mpsc;
use vitals_core::config::AppConfig;

use crate::dbus::{DbusCmd, DbusEvent};
use crate::model::{DaemonState, Snapshot};
use crate::view;

const APP_ID: &str = "com.corecoding.VitalsCosmic";

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Message {
    PopupOpened(window::Id),
    PopupClosed(window::Id),
    Dbus(DbusEvent),
    DbusChannel(mpsc::Sender<DbusCmd>),
    HoverEnter(String),
    HoverLeave,
    ToggleStar(String),
    RefreshNow,
    OpenMonitor,
    OpenPrefs,
    ClosePrefs,
    SetUpdateInterval(u32),
    SetTemperatureUnit(u32),
    SetMemoryMeasurement(u32),
    SetStorageMeasurement(u32),
    SetHigherPrecision(bool),
    SetAlphabetize(bool),
    SetHideZeros(bool),
    SetIncludePublicIp(bool),
    SetMonitorCmd(String),
    RetryDaemon,
    Surface(cosmic::surface::Action),
    Noop,
}

pub struct Vitals {
    core: Core,
    popup: Option<window::Id>,
    prefs_open: bool,

    snapshot: Snapshot,
    history: std::collections::HashMap<String, Vec<(f64, f64)>>,
    /// Monotonically increasing counter used to discard stale
    /// `FetchHistory` responses for hover graphs (see vitals-rs commit 06d529b).
    hover_generation: u64,
    hover_key: Option<String>,

    daemon_state: DaemonState,
    config: AppConfig,
    cmd_tx: Option<mpsc::Sender<DbusCmd>>,
}

impl Vitals {
    fn dispatch(&self, cmd: DbusCmd) {
        if let Some(tx) = &self.cmd_tx {
            let tx = tx.clone();
            tokio::spawn(async move {
                let _ = tx.send(cmd).await;
            });
        }
    }

    fn save_config(&self) {
        if let Ok(toml) = crate::config::to_toml(&self.config) {
            self.dispatch(DbusCmd::SaveConfig(toml));
        }
    }
}

impl Application for Vitals {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Message;
    const APP_ID: &'static str = APP_ID;

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, _flags: Self::Flags) -> (Self, Task<Self::Message>) {
        let state = Self {
            core,
            popup: None,
            prefs_open: false,
            snapshot: Snapshot::default(),
            history: Default::default(),
            hover_generation: 0,
            hover_key: None,
            daemon_state: DaemonState::Connecting,
            config: crate::config::load(),
            cmd_tx: None,
        };
        (state, Task::none())
    }

    fn on_close_requested(&self, id: window::Id) -> Option<Message> {
        Some(Message::PopupClosed(id))
    }

    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            Message::PopupOpened(id) => {
                self.popup = Some(id);
            }
            Message::PopupClosed(id) => {
                if self.popup == Some(id) {
                    self.popup = None;
                    self.prefs_open = false;
                    self.hover_key = None;
                }
            }
            Message::DbusChannel(tx) => {
                self.cmd_tx = Some(tx);
                let secs = self.config.general.update_time.max(1) as u64;
                self.dispatch(DbusCmd::SetInterval(Duration::from_secs(secs)));
            }
            Message::Dbus(event) => match event {
                DbusEvent::State(state) => {
                    self.daemon_state = state;
                }
                DbusEvent::Snapshot(snap) => {
                    self.snapshot = *snap;
                }
                DbusEvent::History { key, points } => {
                    self.history.insert(key, points);
                }
                DbusEvent::SaveResult(ok) => {
                    if !ok {
                        log::warn!("daemon rejected SetConfig");
                    }
                }
            },
            Message::HoverEnter(key) => {
                self.hover_generation = self.hover_generation.wrapping_add(1);
                self.hover_key = Some(key.clone());
                self.dispatch(DbusCmd::FetchHistory(key));
            }
            Message::HoverLeave => {
                self.hover_key = None;
            }
            Message::ToggleStar(key) => {
                if let Some(pos) = self.config.hot_sensors.iter().position(|k| k == &key) {
                    self.config.hot_sensors.remove(pos);
                } else {
                    self.config.hot_sensors.push(key);
                }
                self.save_config();
            }
            Message::RefreshNow => self.dispatch(DbusCmd::RefreshNow),
            Message::OpenMonitor => {
                let cmd = self.config.general.monitor_cmd.clone();
                if !cmd.is_empty() {
                    std::thread::spawn(move || {
                        let _ = std::process::Command::new("sh").arg("-c").arg(&cmd).spawn();
                    });
                }
            }
            Message::OpenPrefs => self.prefs_open = true,
            Message::ClosePrefs => self.prefs_open = false,
            Message::SetUpdateInterval(secs) => {
                self.config.general.update_time = secs.clamp(1, 60);
                self.dispatch(DbusCmd::SetInterval(Duration::from_secs(
                    self.config.general.update_time as u64,
                )));
                self.save_config();
            }
            Message::SetTemperatureUnit(u) => {
                self.config.temperature.unit = u;
                self.save_config();
            }
            Message::SetMemoryMeasurement(m) => {
                self.config.memory.measurement = m;
                self.save_config();
            }
            Message::SetStorageMeasurement(m) => {
                self.config.storage.measurement = m;
                self.save_config();
            }
            Message::SetHigherPrecision(v) => {
                self.config.general.use_higher_precision = v;
                self.save_config();
            }
            Message::SetAlphabetize(v) => {
                self.config.general.alphabetize = v;
                self.save_config();
            }
            Message::SetHideZeros(v) => {
                self.config.general.hide_zeros = v;
                self.save_config();
            }
            Message::SetIncludePublicIp(v) => {
                self.config.network.include_public_ip = v;
                self.save_config();
            }
            Message::SetMonitorCmd(s) => {
                self.config.general.monitor_cmd = s;
                self.save_config();
            }
            Message::RetryDaemon => {
                self.dispatch(DbusCmd::RefreshNow);
            }
            Message::Surface(a) => {
                return cosmic::task::message(cosmic::Action::Cosmic(
                    cosmic::app::Action::Surface(a),
                ));
            }
            Message::Noop => {}
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Self::Message> {
        view::panel::view(self)
    }

    fn view_window(&self, _id: window::Id) -> Element<'_, Self::Message> {
        // Popup content is driven by `app_popup`'s view callback in panel.rs.
        // This fallback only shows if something opens a standalone window.
        cosmic::widget::text::body("").into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::run(|| {
            stream::channel(64, |mut out: cosmic::iced::futures::channel::mpsc::Sender<Message>| async move {
                use cosmic::iced::futures::SinkExt;

                let (ev_tx, mut ev_rx) = mpsc::channel::<DbusEvent>(64);
                let cmd_tx = crate::dbus::spawn(ev_tx, Duration::from_secs(5));
                let _ = out.send(Message::DbusChannel(cmd_tx)).await;

                while let Some(event) = ev_rx.recv().await {
                    if out.send(Message::Dbus(event)).await.is_err() {
                        break;
                    }
                }

                cosmic::iced::futures::future::pending::<()>().await;
                unreachable!()
            })
        })
    }
}

/// Helper used by panel.rs to compute the surface action for opening/closing
/// the popup. Isolated here so `panel::view` stays focused on layout.
pub fn popup_surface_action(
    open: bool,
    existing: Option<window::Id>,
    offset: Vector,
    bounds: Rectangle,
) -> Message {
    if !open {
        if let Some(id) = existing {
            return Message::Surface(destroy_popup(id));
        }
    }

    Message::Surface(app_popup::<Vitals>(
        move |state: &mut Vitals| {
            let new_id = window::Id::unique();
            state.popup = Some(new_id);
            let mut popup_settings = state.core.applet.get_popup_settings(
                state.core.main_window_id().unwrap(),
                new_id,
                None,
                None,
                None,
            );
            popup_settings.positioner.anchor_rect = Rectangle {
                x: (bounds.x - offset.x) as i32,
                y: (bounds.y - offset.y) as i32,
                width: bounds.width as i32,
                height: bounds.height as i32,
            };
            popup_settings.positioner.size_limits = cosmic::iced::Limits::NONE
                .max_width(440.0)
                .min_width(340.0)
                .min_height(160.0)
                .max_height(720.0);
            popup_settings
        },
        Some(Box::new(move |state: &Vitals| {
            Element::from(
                state
                    .core
                    .applet
                    .popup_container(view::popup::view(state)),
            )
            .map(cosmic::Action::App)
        })),
    ))
}

/// Read-only state accessors used by the view modules.
impl Vitals {
    pub fn snapshot(&self) -> &Snapshot {
        &self.snapshot
    }
    pub fn config(&self) -> &AppConfig {
        &self.config
    }
    pub fn daemon_state(&self) -> &DaemonState {
        &self.daemon_state
    }
    pub fn hover_key(&self) -> Option<&str> {
        self.hover_key.as_deref()
    }
    pub fn history(&self, key: &str) -> Option<&[(f64, f64)]> {
        self.history.get(key).map(|v| v.as_slice())
    }
    pub fn core_ref(&self) -> &Core {
        &self.core
    }
    pub fn popup_id(&self) -> Option<window::Id> {
        self.popup
    }
    pub fn prefs_open(&self) -> bool {
        self.prefs_open
    }
}
