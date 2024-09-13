use c8rs_core::{Cpu, EmulatorState};
use ratatui::{
    prelude::*,
    widgets::{block, Block, Borders, Padding, Paragraph},
};

use crate::app::AppState;

use super::Component;

#[derive(Default)]
pub struct CpuComponent {
    focused: bool,
}

impl Component for CpuComponent {
    fn handle_key_event(&mut self, _: crossterm::event::KeyEvent, _: &AppState) -> bool {
        false
    }

    fn render(&mut self, f: &mut Frame<'_>, area: Rect, state: &AppState) {
        let start = std::time::Instant::now();

        let border_style = if self.focused {
            Style::default().fg(Color::Green)
        } else {
            Style::default()
        };

        let outer_block = Block::bordered()
            .title("[2: CPU]")
            .title(block::Title::from(self.render_status_line(state)))
            .border_style(border_style);

        let [reg_area, stack_area] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(Constraint::from_ratios([(1, 2), (1, 2)]))
            .split(outer_block.inner(area))[..]
        else {
            unreachable!()
        };

        f.render_widget(RegisterWidget { state }, reg_area);
        f.render_widget(StackWidget { state }, stack_area);

        f.render_widget(
            outer_block.title(
                block::Title::from(format!(
                    "[render: {:.02}ms]",
                    start.elapsed().as_secs_f64() * 1000.0
                ))
                .alignment(Alignment::Right),
            ),
            area,
        )
    }

    fn has_focus(&self) -> bool {
        self.focused
    }

    fn set_focus(&mut self, focus: bool) {
        self.focused = focus;
    }
}

impl CpuComponent {
    fn render_status_line(&self, state: &AppState) -> String {
        let ips = state.controller.ips();
        format!(
            "[state: {} | IPS: {ips}]",
            match state.controller.state() {
                EmulatorState::Running => "running",
                EmulatorState::Paused => "paused",
                EmulatorState::Halted => "halted",
            }
        )
    }
}

struct RegisterWidget<'a> {
    state: &'a AppState,
}

impl Widget for RegisterWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let block = Block::new()
            .title("registers")
            .title_style(Style::new().bold().underlined())
            .borders(Borders::RIGHT);
        let block_area = block.inner(area);
        block.render(area, buf);

        let Cpu {
            registers,
            delay_timer,
            sound_timer,
            pc,
            sp,
            i,
            ..
        } = self.state.controller.cpu();

        let mut lines = vec![
            Line::from(format!("PC: {pc:#06X}")),
            Line::from(format!("SP: {sp:#06X}")),
            Line::from(format!("I:  {i:#06X}")),
            Line::from(""),
            Line::from(format!("DT: {delay_timer:#04X} ({delay_timer:03})")),
            Line::from(format!("ST: {sound_timer:#04X} ({sound_timer:03})")),
            Line::from(""),
        ];

        let fmt_reg = |i, val| format!("V{i:X}: {val:#04X} ({val:03})");
        lines.extend((0x0..0x8usize).map(|i| {
            let j = i + 8;
            let val_i = registers[i];
            let val_j = registers[j];

            Line::from(vec![
                Span::from(fmt_reg(i, val_i)),
                Span::from(" ".repeat(8)),
                Span::from(fmt_reg(j, val_j)),
            ])
        }));

        Paragraph::new(Text::from(lines)).render(block_area, buf);
    }
}

struct StackWidget<'a> {
    state: &'a AppState,
}

impl Widget for StackWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let block = Block::new()
            .title("stack")
            .title_style(Style::new().bold().underlined())
            .padding(Padding::top(2));
        let block_area = block.inner(area);
        block.render(area, buf);

        let Cpu { sp, .. } = self.state.controller.cpu();

        let mem = self.state.controller.memory();

        let fmt_addr = |addr| {
            let val = mem.read_u16(addr);
            format!(
                "{}|{addr:#06X}| {val:#06X}",
                if *sp == addr { "SP->" } else { "    " }
            )
        };

        let lines = (0..8).map(|row| {
            let addr = 0x1E0 + (row as u16 * 2);
            let addr_2 = addr + 16;
            Line::from(vec![
                Span::from(fmt_addr(addr)),
                Span::raw("  │ "),
                Span::from(fmt_addr(addr_2)),
            ])
        });

        Paragraph::new(Text::from_iter(lines)).render(block_area, buf);
    }
}
