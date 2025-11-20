use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::Span;
use ratatui::widgets::Gauge;

use crate::runtime::GaugeView;

pub fn render_gauge(frame: &mut Frame<'_>, area: Rect, view: &GaugeView) {
    let mut widget = Gauge::default()
        .use_unicode(true)
        .ratio(view.ratio.clamp(0.0, 1.0));

    if let Some(label) = &view.label {
        widget = widget.label(Span::raw(label.clone()));
    } else {
        let percent = (view.ratio * 100.0).round();
        widget = widget.label(Span::raw(format!("{percent:.0}%")));
    }

    if let Some(color) = view.color {
        widget = widget.style(Style::default().fg(color));
    }

    frame.render_widget(widget, area);
}
