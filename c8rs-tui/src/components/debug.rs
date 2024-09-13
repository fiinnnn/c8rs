use c8rs_core::{DebugCommand, EmulatorCommand};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    prelude::*,
    widgets::{block, Block},
};

use crate::app::AppState;

use super::Component;

#[derive(Default)]
pub struct DebuggerComponent {
    focused: bool,

    history: Vec<String>,
    input: String,
    cursor_pos: usize,
}

impl Component for DebuggerComponent {
    fn handle_key_event(&mut self, event: KeyEvent, state: &AppState) -> bool {
        match event.code {
            KeyCode::Char(c) => {
                self.insert_char(c);
            }
            KeyCode::Backspace => {
                self.delete_char();
            }
            KeyCode::Enter => {
                self.submit(state);
            }
            KeyCode::Right => {
                self.move_cursor_right();
            }
            KeyCode::Left => {
                self.move_cursor_left();
            }
            _ => return false,
        }
        true
    }

    fn render(&mut self, f: &mut ratatui::Frame<'_>, area: Rect, state: &AppState) {
        let start = std::time::Instant::now();

        let border_style = if self.focused {
            Style::default().fg(Color::Green)
        } else {
            Style::default()
        };

        let outer_block = Block::bordered()
            .title("[6: Debugger]")
            .border_style(border_style);

        let inner_area = outer_block.inner(area);

        let [history_area, input_area] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Fill(1), Constraint::Length(1)])
            .split(inner_area)[..]
        else {
            unreachable!()
        };

        let history = Text::from_iter((0..history_area.height as isize).rev().map(|i| {
            let i = self.history.len() as isize - i - 1;
            if i < 0 {
                Line::default()
            } else {
                Line::from(self.history[i as usize].clone())
            }
        }));

        let input_line = Line::from(self.input.to_string());
        let cursor_pos = Position::new(input_area.x + self.cursor_pos as u16, input_area.y);

        f.render_widget(history, history_area);
        f.render_widget(input_line, input_area);

        if self.focused {
            f.set_cursor_position(cursor_pos);
        }

        f.render_widget(
            outer_block.title(
                block::Title::from(format!(
                    "[render: {:.02}ms]",
                    start.elapsed().as_secs_f64() * 1000.0
                ))
                .alignment(Alignment::Right),
            ),
            area,
        );
    }

    fn has_focus(&self) -> bool {
        self.focused
    }

    fn set_focus(&mut self, focus: bool) {
        self.focused = focus
    }
}

impl DebuggerComponent {
    fn move_cursor_right(&mut self) {
        self.cursor_pos = self
            .cursor_pos
            .saturating_add(1)
            .clamp(0, self.input.chars().count());
    }

    fn move_cursor_left(&mut self) {
        self.cursor_pos = self
            .cursor_pos
            .saturating_sub(1)
            .clamp(0, self.input.chars().count());
    }

    fn insert_char(&mut self, c: char) {
        let i = self
            .input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.cursor_pos)
            .unwrap_or(self.input.len());

        self.input.insert(i, c);
        self.move_cursor_right();
    }

    fn delete_char(&mut self) {
        if self.cursor_pos == 0 {
            return;
        }

        let i = self.cursor_pos;
        let before = self.input.chars().take(i - 1);
        let after = self.input.chars().skip(i);

        self.input = before.chain(after).collect();
        self.move_cursor_left();
    }

    fn submit(&mut self, state: &AppState) {
        let input = self.input.clone();
        self.history.push(input.clone());
        self.input.clear();
        self.cursor_pos = 0;

        let cmd = match DebugCommand::parse_from(&input) {
            Ok(cmd) => cmd,
            Err(err) => {
                for line in err.lines() {
                    self.history.push(line.to_string());
                }
                return;
            }
        };

        let _ = state.controller.send(EmulatorCommand::DebugCommand(cmd));
    }
}
