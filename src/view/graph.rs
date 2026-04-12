//! Hover history view. First iteration renders a compact numeric summary
//! (min / avg / max / last) instead of a canvas graph, because iced's
//! `canvas::Program` trait has generic bounds that need careful pinning
//! against the in-tree libcosmic commit. A Canvas-based graph is a follow-up.

use cosmic::iced::Length;
use cosmic::widget::{container, Column, Row};
use cosmic::Element;

use crate::app::Message;

pub fn view(points: Vec<(f64, f64)>, height: f32) -> Element<'static, Message> {
    if points.is_empty() {
        return container(cosmic::widget::text::body("No history yet…"))
            .padding(6)
            .height(Length::Fixed(height))
            .into();
    }

    let mut min = f64::INFINITY;
    let mut max = f64::NEG_INFINITY;
    let mut sum = 0.0;
    for (_, v) in &points {
        if *v < min {
            min = *v;
        }
        if *v > max {
            max = *v;
        }
        sum += v;
    }
    let avg = sum / points.len() as f64;
    let last = points.last().map(|p| p.1).unwrap_or(0.0);

    let summary = format!(
        "min {:.2}   avg {:.2}   max {:.2}   last {:.2}",
        min, avg, max, last
    );

    // Cheap textual sparkline so you can at least see shape at a glance.
    let sparkline = sparkline(&points, 60);

    container(
        Column::new()
            .spacing(4)
            .push(Row::new().push(cosmic::widget::text::body(sparkline)))
            .push(Row::new().push(cosmic::widget::text::caption(summary))),
    )
    .padding(6)
    .width(Length::Fill)
    .into()
}

fn sparkline(points: &[(f64, f64)], width: usize) -> String {
    const BARS: &[char] = &['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

    let values: Vec<f64> = points.iter().map(|p| p.1).collect();
    let start = values.len().saturating_sub(width);
    let slice = &values[start..];

    let min = slice.iter().copied().fold(f64::INFINITY, f64::min);
    let max = slice.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let range = (max - min).max(f64::EPSILON);

    slice
        .iter()
        .map(|v| {
            let norm = ((v - min) / range).clamp(0.0, 1.0);
            let idx = (norm * (BARS.len() - 1) as f64).round() as usize;
            BARS[idx]
        })
        .collect()
}
