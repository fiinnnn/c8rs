use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{prelude::*, widgets::Block};
use tui_logger::TuiLoggerWidget;

use crate::app::AppState;

use super::Component;

#[derive(Default)]
pub struct LogComponent {
    focused: bool,
    state: tui_logger::TuiWidgetState,
}

impl Component for LogComponent {
    fn handle_key_event(&mut self, event: KeyEvent, _: &AppState) -> bool {
        let log_event = match event.code {
            KeyCode::Char('j') => tui_logger::TuiWidgetEvent::NextPageKey,
            KeyCode::Char('k') => tui_logger::TuiWidgetEvent::PrevPageKey,
            _ => return false,
        };

        self.state.transition(log_event);

        true
    }

    fn render(&mut self, f: &mut Frame<'_>, area: Rect, _: &AppState) {
        let border_style = if self.focused {
            Style::default().fg(Color::Green)
        } else {
            Style::default()
        };

        f.render_widget(
            TuiLoggerWidget::default()
                .block(Block::bordered().title("[5: Log]").style(border_style))
                .output_separator('|')
                .output_timestamp(Some("%H:%M:%S%.3f".to_string()))
                .style_error(Style::default().fg(Color::Red))
                .style_debug(Style::default().fg(Color::Green))
                .style_warn(Style::default().fg(Color::Yellow))
                .style_trace(Style::default().fg(Color::Magenta))
                .style_info(Style::default().fg(Color::Cyan))
                .state(&self.state),
            area,
        );
    }

    fn has_focus(&self) -> bool {
        self.focused
    }

    fn set_focus(&mut self, focus: bool) {
        self.focused = focus;

        if !focus {
            self.state.transition(tui_logger::TuiWidgetEvent::EscapeKey);
        }
    }
}
