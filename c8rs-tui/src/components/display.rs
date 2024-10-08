use ratatui::{
    prelude::*,
    widgets::{block, Block},
};

use crate::app::AppState;

use super::Component;

#[derive(Default)]
pub struct DisplayComponent {
    focused: bool,
}

impl Component for DisplayComponent {
    fn handle_key_event(&mut self, _event: crossterm::event::KeyEvent, _: &AppState) -> bool {
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
            .title("[1: CHIP-8]")
            .border_style(border_style);
        let block_area = outer_block.inner(area);

        let display = state.controller.display();
        let (width, _) = display.get_dimensions();
        let pixels = display.get_pixels();

        f.render_widget(
            DisplayWidget {
                pixels: &pixels,
                width,
            },
            block_area,
        );

        f.render_widget(
            outer_block.title(
                block::Title::from(format!(
                    "[render: {:.02} ms]",
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

struct DisplayWidget<'a> {
    pixels: &'a [bool],
    width: usize,
}

impl Widget for DisplayWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        for (i, pixel) in self.pixels.iter().enumerate() {
            let x = i % self.width;
            let y = i / self.width;

            let Some(cell) = buf.cell_mut((area.left() + x as u16, area.top() + (y / 2) as u16))
            else {
                continue;
            };

            let color = if *pixel { Color::White } else { Color::Black };

            if y % 2 == 0 {
                cell.set_bg(color);
            } else {
                cell.set_fg(color).set_symbol("▄");
            }
        }
    }
}
