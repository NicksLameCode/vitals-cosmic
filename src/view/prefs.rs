//! Preferences sub-popup. Reuses the same popup surface as `view::popup` but
//! `Vitals::prefs_open()` decides which content to render.

use cosmic::iced::Length;
use cosmic::widget::{button, container, divider, settings, text_input, toggler, Column, Row};
use cosmic::Element;

use crate::app::{Message, Vitals};

pub fn view(state: &Vitals) -> Element<'_, Message> {
    let cfg = state.config();

    let header = Row::new()
        .spacing(8)
        .push(cosmic::widget::text::heading("Preferences"))
        .push(button::text("Done").on_press(Message::ClosePrefs));

    let general = settings::section()
        .title("General")
        .add(settings::item(
            "Higher precision values",
            toggler(cfg.general.use_higher_precision).on_toggle(Message::SetHigherPrecision),
        ))
        .add(settings::item(
            "Alphabetize sensors",
            toggler(cfg.general.alphabetize).on_toggle(Message::SetAlphabetize),
        ))
        .add(settings::item(
            "Hide zero values",
            toggler(cfg.general.hide_zeros).on_toggle(Message::SetHideZeros),
        ))
        .add(settings::item(
            "System monitor command",
            text_input("btop", &cfg.general.monitor_cmd)
                .on_input(Message::SetMonitorCmd)
                .width(Length::Fixed(220.0)),
        ));

    let units = settings::section()
        .title("Units")
        .add(settings::item(
            "Temperature",
            Row::new()
                .spacing(4)
                .push(button::text("°C").on_press(Message::SetTemperatureUnit(0)))
                .push(button::text("°F").on_press(Message::SetTemperatureUnit(1))),
        ))
        .add(settings::item(
            "Memory",
            Row::new()
                .spacing(4)
                .push(button::text("GB").on_press(Message::SetMemoryMeasurement(1)))
                .push(button::text("GiB").on_press(Message::SetMemoryMeasurement(0))),
        ))
        .add(settings::item(
            "Storage",
            Row::new()
                .spacing(4)
                .push(button::text("GB").on_press(Message::SetStorageMeasurement(1)))
                .push(button::text("GiB").on_press(Message::SetStorageMeasurement(0))),
        ));

    let network = settings::section().title("Network").add(settings::item(
        "Include public IP",
        toggler(cfg.network.include_public_ip).on_toggle(Message::SetIncludePublicIp),
    ));

    let interval = settings::section().title("Polling").add(settings::item(
        format!("Update interval: {} s", cfg.general.update_time),
        Row::new()
            .spacing(4)
            .push(button::text("1s").on_press(Message::SetUpdateInterval(1)))
            .push(button::text("2s").on_press(Message::SetUpdateInterval(2)))
            .push(button::text("5s").on_press(Message::SetUpdateInterval(5)))
            .push(button::text("10s").on_press(Message::SetUpdateInterval(10))),
    ));

    container(
        Column::new()
            .spacing(8)
            .push(header)
            .push(divider::horizontal::default())
            .push(interval)
            .push(general)
            .push(units)
            .push(network),
    )
    .padding(8)
    .into()
}
