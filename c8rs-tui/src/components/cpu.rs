use c8rs_core::{Cpu, EmulatorState, Memory};
use ratatui::{
    prelude::*,
    widgets::{block, Block},
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

        let cpu = state.controller.cpu();
        let mem = state.controller.memory();

        f.render_widget(RegisterWidget { cpu }, reg_area);
        f.render_widget(StackWidget { cpu, mem }, stack_area);

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
    cpu: &'a Cpu,
}

impl Widget for RegisterWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let Cpu {
            pc,
            sp,
            i,
            delay_timer,
            sound_timer,
            registers,
            ..
        } = self.cpu;

        buf.set_line(
            area.x,
            area.y,
            &Line::from(format!("PC: {pc:#06X}")),
            area.width,
        );
        buf.set_line(
            area.x,
            area.y + 1,
            &Line::from(format!("SP: {sp:#06X}")),
            area.width,
        );
        buf.set_line(
            area.x,
            area.y + 2,
            &Line::from(format!("I:  {i:#06X}")),
            area.width,
        );

        buf.set_line(
            area.x,
            area.y + 4,
            &Line::from(format!("DT: {delay_timer:#04X} ({delay_timer:03})")),
            area.width,
        );
        buf.set_line(
            area.x,
            area.y + 5,
            &Line::from(format!("ST: {sound_timer:#04X} ({sound_timer:03})")),
            area.width,
        );

        for col in 0..2 {
            for row in 0..8 {
                let reg = row + col * 8;
                let val = registers[reg];

                buf.set_span(
                    area.x + (col as u16 * 22),
                    area.y + 7 + row as u16,
                    &Span::from(format!("V{reg:X}: {val:#04X} ({val:03})")),
                    area.width,
                );
            }
        }
    }
}

struct StackWidget<'a> {
    cpu: &'a Cpu,
    mem: &'a Memory,
}

impl Widget for StackWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let Cpu { sp, .. } = self.cpu;

        let stack_len = 0x1FEu16.saturating_sub(*sp) / 2;
        buf.set_line(
            area.x,
            area.y,
            &Line::from(format!("stack    len: {stack_len}")),
            area.width,
        );

        let start_addr = sp.saturating_sub(area.height).min(0x0202 - area.height * 2) & 0xFFFE;
        let end_addr = (start_addr + area.height * 2) & 0xFFFE;

        for (i, addr) in (start_addr..end_addr).step_by(2).enumerate() {
            let val = self.mem.read_u16(addr);
            buf.set_line(
                area.x,
                area.y + 1 + i as u16,
                &Line::from(format!(
                    "{}|{addr:#06X}| {val:#06X}",
                    if *sp == addr { "SP->" } else { "    " }
                )),
                area.width,
            );
        }
    }
}
