use c8rs_core::{Cpu, DebugCommand, EmulatorCommand, Instruction};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    prelude::*,
    widgets::{block, Block, Paragraph},
};

use crate::app::AppState;

use super::Component;

#[derive(Default)]
pub struct DisassemblyComponent {
    focused: bool,
    mode: Mode,
    addr: u16,
    input: String,
    prev_mode: Mode,
}

#[derive(Default, Copy, Clone, PartialEq)]
enum Mode {
    #[default]
    Follow,
    Manual,
    GotoInput,
}

impl Component for DisassemblyComponent {
    fn handle_key_event(&mut self, event: KeyEvent, state: &AppState) -> bool {
        match self.mode {
            Mode::Follow | Mode::Manual => {
                match event.code {
                    KeyCode::Char('f') => self.mode = Mode::Follow,
                    KeyCode::Char('j') => {
                        self.mode = Mode::Manual;
                        self.addr = self.addr.saturating_add(2);
                    }
                    KeyCode::Char('k') => {
                        self.mode = Mode::Manual;
                        self.addr = self.addr.saturating_sub(2);
                    }
                    KeyCode::Char('b') => {
                        let _ = state.controller.send(EmulatorCommand::DebugCommand(
                            DebugCommand::Breakpoint { addr: self.addr },
                        ));
                    }
                    KeyCode::Char('g') => {
                        self.prev_mode = self.mode;
                        self.mode = Mode::GotoInput;
                        self.input.clear();
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
                        self.mode = self.prev_mode;
                    }
                    KeyCode::Enter => {
                        self.mode = Mode::Manual;
                        let input = self.input.trim_start_matches("0x");
                        if let Ok(addr) = u16::from_str_radix(input, 16) {
                            self.addr = addr;
                        }
                    }
                    _ => return false,
                }
                true
            }
        }
    }

    fn render(&mut self, f: &mut Frame<'_>, area: Rect, state: &AppState) {
        let start = std::time::Instant::now();

        let border_style = if self.focused {
            Style::default().fg(Color::Green)
        } else {
            Style::default()
        };

        let outer_block = Block::bordered()
            .title("[3: Disassembly]")
            .title(
                block::Title::from(self.render_status_line())
                    .position(block::Position::Bottom)
                    .alignment(Alignment::Right),
            )
            .border_style(border_style);
        let block_area = outer_block.inner(area);

        let Cpu { pc, .. } = state.controller.cpu();
        let mem = state.controller.memory();
        let breakpoints = state.controller.breakpoints();

        if self.mode == Mode::Follow {
            self.addr = *pc;
        }

        let format_addr = |addr| {
            let word = mem.read_u16(addr);
            let high_byte = (word >> 8) as u8;
            let low_byte = (word & 0xFF) as u8;
            let inst = Instruction::parse(word);

            let line_style = if addr == *pc {
                Style::new().black().on_green()
            } else if self.mode == Mode::Manual && addr == self.addr {
                Style::new().black().on_blue()
            } else {
                Style::default()
            };

            let breakpoint = if breakpoints.contains(&addr) {
                "b"
            } else {
                " "
            };

            let mut lines = vec![
                Span::from(format!("{breakpoint} ")),
                Span::from(format!("{addr:#06X}:")),
                Span::from("  "),
                Span::from(format!("{high_byte:02X}")),
                Span::from(" "),
                Span::from(format!("{low_byte:02X}")),
                Span::from("  "),
                Span::from(format!("{inst}")),
            ];

            let content_len = lines.iter().fold(0, |len, l| len + l.content.len());
            if block_area.width as usize > content_len {
                lines.push(Span::from(
                    " ".repeat(block_area.width as usize - content_len),
                ));
            }

            Line::from(lines).style(line_style)
        };

        let start_addr = (self.addr.saturating_sub(block_area.height))
            .min(0x1000 - block_area.height * 2)
            & 0xFFFE;
        let end_addr = (start_addr + block_area.height * 2) & 0xFFFE;
        let lines = (start_addr..end_addr).step_by(2).map(format_addr);

        f.render_widget(Paragraph::new(Text::from_iter(lines)), block_area);

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

impl DisassemblyComponent {
    fn render_status_line(&self) -> String {
        match self.mode {
            Mode::Follow => "[addr: PC]".to_string(),
            Mode::Manual => format!("[addr: {:#06X}]", self.addr),
            Mode::GotoInput => format!("[goto: {}]", self.input),
        }
    }
}
