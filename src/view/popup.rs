//! Dropdown popup: category sections, each listing sensors with a star toggle.

use cosmic::iced::{alignment, Length};
use cosmic::widget::{button, container, divider, mouse_area, scrollable, Column, Row};
use cosmic::Element;

use crate::app::{Message, Vitals};
use crate::format::format_reading;
use crate::icons;
use crate::model::{Category, DaemonState, Reading, ReadingKind};
use crate::view::graph;
use crate::view::prefs;

pub fn view(state: &Vitals) -> Element<'_, Message> {
    if state.prefs_open() {
        return prefs::view(state);
    }

    let cfg = state.config();
    let snap = state.snapshot();
    let grouped = snap.grouped();

    let mut body = Column::new().spacing(8).width(Length::Fill);

    // Daemon state header.
    match state.daemon_state() {
        DaemonState::Unreachable(reason) => {
            body = body.push(
                container(
                    Row::new()
                        .spacing(6)
                        .push(cosmic::widget::text::body(format!(
                            "Daemon unreachable: {reason}"
                        )))
                        .push(button::text("Retry").on_press(Message::RetryDaemon)),
                )
                .padding(6),
            );
        }
        DaemonState::Connecting => {
            body = body.push(cosmic::widget::text::body("Connecting to vitals-daemon…"));
        }
        DaemonState::Connected => {}
    }

    // Categories in stable order, then dynamic GPUs at the end.
    let mut order: Vec<Category> = Category::display_order().to_vec();
    for cat in grouped.keys() {
        if matches!(cat, Category::Gpu(_)) && !order.contains(cat) {
            order.push(*cat);
        }
    }

    for category in order {
        let Some(readings) = grouped.get(&category) else {
            continue;
        };
        if readings.is_empty() {
            continue;
        }

        let header = Row::new()
            .spacing(6)
            .align_y(alignment::Vertical::Center)
            .push(cosmic::widget::icon(icons::for_category(category.icon_name())).size(16))
            .push(cosmic::widget::text::heading(category.title()));

        let mut rows = Column::new().spacing(2);
        for reading in readings {
            if cfg.general.hide_zeros && is_zero(reading) {
                continue;
            }
            rows = rows.push(sensor_row(state, reading));
        }

        body = body
            .push(header)
            .push(rows)
            .push(divider::horizontal::default());
    }

    // Hover graph.
    if let Some(key) = state.hover_key() {
        if let Some(points) = state.history(key) {
            body = body.push(graph::view(points.to_vec(), 160.0));
        }
    }

    let footer = Row::new()
        .spacing(8)
        .push(button::text("Refresh").on_press(Message::RefreshNow))
        .push(button::text("Open monitor").on_press(Message::OpenMonitor))
        .push(button::text("Preferences").on_press(Message::OpenPrefs));

    let content = Column::new()
        .spacing(8)
        .push(
            scrollable(body)
                .height(Length::Fixed(500.0))
                .width(Length::Fill),
        )
        .push(divider::horizontal::default())
        .push(footer);

    container(content).padding(8).into()
}

fn sensor_row<'a>(state: &'a Vitals, reading: &'a Reading) -> Element<'a, Message> {
    let pinned = state
        .config()
        .hot_sensors
        .iter()
        .any(|k| k == &reading.key);
    let star = if pinned { "★" } else { "☆" };

    let value_text = format_reading(state.config(), reading);

    let inner = Row::new()
        .spacing(6)
        .align_y(alignment::Vertical::Center)
        .push(cosmic::widget::text::body(reading.label.clone()).width(Length::FillPortion(3)))
        .push(cosmic::widget::text::body(value_text).width(Length::FillPortion(2)))
        .push(button::text(star).on_press(Message::ToggleStar(reading.key.clone())));

    let key = reading.key.clone();
    mouse_area(container(inner).padding(4))
        .on_enter(Message::HoverEnter(key))
        .on_exit(Message::HoverLeave)
        .into()
}

fn is_zero(reading: &Reading) -> bool {
    matches!(
        reading.kind,
        ReadingKind::Numeric { value, .. } if value.abs() < f64::EPSILON
    )
}
