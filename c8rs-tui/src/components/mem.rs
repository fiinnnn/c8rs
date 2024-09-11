use c8rs_core::{Cpu, Memory};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    prelude::*,
    widgets::{block, Block},
};

use crate::app::AppState;

use super::Component;

#[derive(Default)]
pub struct MemoryComponent {
    focused: bool,
    offset: u16,
    mode: Mode,
    view: View,
    input: String,
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
enum Mode {
    #[default]
    Normal,
    GotoInput,
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
enum View {
    #[default]
    Hex,
    Sprite,
}

impl Component for MemoryComponent {
    fn handle_key_event(&mut self, event: KeyEvent, state: &AppState) -> bool {
        match self.mode {
            Mode::Normal => {
                match event.code {
                    KeyCode::Char('j') => {
                        let diff = if self.view == View::Sprite { 1 } else { 16 };
                        self.offset = self.offset.saturating_add(diff).min(0xFF0)
                    }
                    KeyCode::Char('k') => {
                        let diff = if self.view == View::Sprite { 1 } else { 16 };
                        self.offset = self.offset.saturating_sub(diff)
                    }
                    KeyCode::Char('g') => {
                        self.mode = Mode::GotoInput;
                        self.input.clear();
                    }
                    KeyCode::Char('i') => {
                        let Cpu { i, .. } = state.controller.cpu();
                        self.offset = i & 0xFF0;
                    }
                    KeyCode::Char('s') => self.view = View::Sprite,
                    KeyCode::Char('h') => {
                        self.view = View::Hex;
                        self.offset &= 0xFF0;
                    }
                    _ => return false,
                }
                true
            }
            Mode::GotoInput => {
                match event.code {
                    KeyCode::Char(c) => self.input.push(c),
                    KeyCode::Backspace => {
                        self.input.pop();
                    }
                    KeyCode::Esc => {
                        self.mode = Mode::Normal;
                    }
                    KeyCode::Enter => {
                        self.mode = Mode::Normal;
                        let input = self.input.trim_start_matches("0x");
                        if let Ok(offset) = u16::from_str_radix(input, 16) {
                            self.offset = offset;
                        }
                    }
                    _ => return false,
                }
                true
            }
        }
    }

    fn render(&mut self, f: &mut Frame<'_>, area: Rect, state: &AppState) {
        let border_style = if self.focused {
            Style::default().fg(Color::Green)
        } else {
            Style::default()
        };

        let outer_block = Block::bordered()
            .title("[4: Memory]")
            .title(
                block::Title::from(self.render_status_line())
                    .position(block::Position::Bottom)
                    .alignment(Alignment::Right),
            )
            .border_style(border_style);
        let block_area = outer_block.inner(area);

        let cpu = state.controller.cpu();
        let mem = state.controller.memory();

        f.render_widget(outer_block, area);
        match self.view {
            View::Hex => f.render_widget(self.render_hex(cpu, mem, block_area.height), block_area),
            View::Sprite => {
                f.render_widget(self.render_sprite(cpu, mem, block_area.height), block_area)
            }
        }
    }

    fn has_focus(&self) -> bool {
        self.focused
    }

    fn set_focus(&mut self, focus: bool) {
        self.focused = focus
    }
}

impl MemoryComponent {
    fn render_status_line(&self) -> String {
        let view = match self.view {
            View::Hex => "view: hex",
            View::Sprite => "view: sprite",
        };

        match self.mode {
            Mode::Normal => format!("[{view} | offset: {:#06X}]", self.offset),
            Mode::GotoInput => format!("[{view} | goto: {}]", self.input),
        }
    }

    fn render_hex(&self, cpu: &Cpu, mem: &Memory, height: u16) -> Text {
        let Cpu { pc, sp, i, .. } = cpu;
        let lines = (0..height).map(|row| {
            if row == 0 {
                return Line::from(
                    "             0  1  2  3  4  5  6  7  8  9  A  B  C  D  E  F".to_string(),
                );
            }

            let offset = self.offset + ((row - 1) * 16);
            if offset > 0xFF0 {
                return Line::default();
            }

            let mut spans = Vec::with_capacity(19);

            let row_has_pc = offset == *pc & 0xFF0;
            let row_has_sp = offset == *sp & 0xFF0;
            let row_has_i = offset == *i & 0xFF0;

            spans.push(if row_has_pc {
                Span::styled("PC ", Style::default().fg(Color::Yellow))
            } else if row_has_sp {
                Span::styled("SP ", Style::default().fg(Color::Magenta))
            } else if row_has_i {
                Span::styled(" I ", Style::default().fg(Color::Green))
            } else {
                Span::raw("   ")
            });

            spans.push(Span::from(format!("|{offset:#06X}|")));

            for byte_offset in 0..16 {
                let addr = offset + byte_offset;
                let byte = mem.read_u8(addr);

                let style = if addr.saturating_sub(1) == *pc || addr == *pc {
                    Style::default().fg(Color::Yellow)
                } else if addr.saturating_sub(1) == *sp || addr == *sp {
                    Style::default().fg(Color::Magenta)
                } else if addr.saturating_sub(1) == *i || addr == *i {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default()
                };

                spans.push(Span::styled(format!(" {byte:02X}"), style));
            }

            Line::from(spans)
        });

        Text::from_iter(lines)
    }

    fn render_sprite(&self, cpu: &Cpu, mem: &Memory, height: u16) -> Text {
        let Cpu { i, .. } = cpu;
        Text::from_iter((0..height).map(|row| {
            let addr = self.offset + row;

            let i_str = if *i == addr { "I" } else { " " };
            let mut spans = vec![Span::styled(
                format!(" {i_str} |{addr:#06X}| "),
                if *i == addr {
                    Style::new().green()
                } else {
                    Style::default()
                },
            )];

            let byte = mem.read_u8(addr);
            for j in 0..8 {
                if (byte >> (7 - j)) & 0x1 == 1 {
                    spans.push(Span::styled("█", Style::new().white()))
                } else {
                    spans.push(Span::styled("█", Style::new().black()))
                }
            }

            Line::from(spans)
        }))
    }
}
