//! Compact panel view. Layout recipe mirrors `cosmic-applet-time`:
//! `button::custom` wrapped in `widget::autosize::autosize(btn, AUTOSIZE_MAIN_ID)`
//! so cosmic-panel sizes the slot to fit the content.

use std::rc::Rc;
use std::sync::LazyLock;

use cosmic::iced::alignment::Vertical;
use cosmic::iced::Length;
use cosmic::widget::{autosize, button, container, space, svg, Id, Row};
use cosmic::Element;

use crate::app::{popup_surface_action, Message, Vitals};
use crate::format::format_reading;
use crate::icons;
use crate::model::ReadingKind;

static AUTOSIZE_MAIN_ID: LazyLock<Id> = LazyLock::new(|| Id::new("autosize-main"));

pub fn view(state: &Vitals) -> Element<'_, Message> {
    let cfg = state.config();
    let snap = state.snapshot();
    let applet = &state.core_ref().applet;

    let (icon_major, icon_minor) = applet.suggested_size(true);
    let (h_pad, v_pad) = applet.suggested_padding(true);
    let is_horizontal = applet.is_horizontal();

    // Build the chip row: one (icon, value) pair per resolved hot sensor.
    let mut chips = Row::new().spacing(6).align_y(Vertical::Center);
    let icon_px = icon_major.saturating_sub(2).max(12);

    let mut rendered = 0usize;
    for key in &cfg.hot_sensors {
        if rendered >= 4 {
            break;
        }
        let Some(reading) = snap.readings.get(key) else {
            continue;
        };
        if matches!(reading.kind, ReadingKind::Text(_)) {
            continue;
        }

        chips = chips
            .push(symbolic_icon(reading.category.icon_name(), icon_px))
            .push(applet.text(format_reading(cfg, reading)));
        rendered += 1;
    }

    // Fallback: if the daemon hasn't delivered any starred sensors yet (or the
    // user hasn't pinned any), show a single hint so the applet still has a
    // hit-target in the panel.
    if rendered == 0 {
        chips = chips.push(applet.text("Vitals"));
    }

    // Vertical spacer so the button height matches the other applets in the row.
    let spacer_height = icon_minor + 2 * v_pad;
    let laid_out = Row::new()
        .align_y(Vertical::Center)
        .push(chips)
        .push(
            container(space::vertical().height(Length::Fixed(spacer_height as f32)))
                .width(Length::Shrink),
        );

    let padding = if is_horizontal {
        [0, h_pad]
    } else {
        [h_pad, 0]
    };

    let have_popup = state.popup_id();
    let btn = button::custom(laid_out)
        .padding(padding)
        .class(cosmic::theme::Button::AppletIcon)
        .on_press_with_rectangle(move |offset, bounds| {
            popup_surface_action(have_popup.is_none(), have_popup, offset, bounds)
        });

    autosize::autosize(btn, AUTOSIZE_MAIN_ID.clone()).into()
}

/// Render an embedded SVG with the panel's foreground color applied, matching
/// how `applet.icon_button_from_handle` themes symbolic icons.
fn symbolic_icon(name: &str, size: u16) -> Element<'static, Message> {
    cosmic::widget::icon(icons::for_category(name))
        .class(cosmic::theme::Svg::Custom(Rc::new(|theme| svg::Style {
            color: Some(theme.cosmic().background.on.into()),
        })))
        .size(size)
        .into()
}
