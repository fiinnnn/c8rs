use std::time::Duration;

use anyhow::Result;
use c8rs_core::{EmulatorCommand, EmulatorController};
use crossterm::event::KeyEvent;
use futures::{FutureExt, StreamExt};
use ratatui::{
    crossterm::event::{Event, KeyCode, KeyEventKind},
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders},
    Frame,
};
use tokio_util::sync::CancellationToken;

use crate::{
    components::{
        Component, CpuComponent, DebuggerComponent, DisassemblyComponent, DisplayComponent,
        LogComponent, MemoryComponent,
    },
    tui,
};

pub struct App {
    state: AppState,
    cancellation_token: CancellationToken,
    panels: Vec<Box<dyn Component>>,
}

#[derive(Debug, Clone)]
enum AppEvent {
    Tick,
    Render,
    Key(KeyEvent),
    Error(String),
}

pub struct AppState {
    pub controller: EmulatorController,
}

impl App {
    pub fn new(controller: EmulatorController) -> Self {
        App {
            state: AppState { controller },
            cancellation_token: CancellationToken::new(),
            panels: vec![
                Box::new(DisplayComponent::default()),
                Box::new(CpuComponent::default()),
                Box::new(DisassemblyComponent::default()),
                Box::new(MemoryComponent::default()),
                Box::new(LogComponent::default()),
                Box::new(DebuggerComponent::default()),
            ],
        }
    }

    pub fn init_logger() {
        tui_logger::init_logger(log::LevelFilter::Debug).unwrap();
        tui_logger::set_default_level(log::LevelFilter::Debug);
    }

    pub async fn run(&mut self) -> Result<()> {
        let mut terminal = tui::init()?;

        let (event_tx, mut event_rx) = tokio::sync::mpsc::unbounded_channel();

        let cancellation_token = self.cancellation_token.clone();
        tokio::spawn(async move {
            let mut tick_interval = tokio::time::interval(Duration::from_secs_f64(1.0 / 4.0));
            let mut event_stream = crossterm::event::EventStream::new();

            loop {
                let tick_delay = tick_interval.tick();
                let event = event_stream.next().fuse();

                tokio::select! {
                    _ = cancellation_token.cancelled() => {
                        break;
                    }
                    _ = tick_delay => {
                        event_tx.send(AppEvent::Tick).unwrap();
                    },
                    event_opt = event => {
                        match event_opt {
                            Some(Ok(event)) => {
                                match event {
                                    Event::Key(key) => {
                                        if key.kind == KeyEventKind::Press {
                                            event_tx.send(AppEvent::Key(key)).unwrap();
                                        }
                                    },
                                    Event::Resize(_, _) => event_tx.send(AppEvent::Render).unwrap(),
                                    _ => (),
                                }
                            },
                            Some(Err(err)) => {
                                event_tx.send(AppEvent::Error(err.to_string())).unwrap();
                            }
                            None => (),
                        }
                    }
                }
            }
        });

        while !self.cancellation_token.is_cancelled() {
            while let Ok(event) = event_rx.try_recv() {
                match event {
                    AppEvent::Tick => (),
                    AppEvent::Key(key) => self.handle_key_event(key),
                    AppEvent::Error(err) => log::error!("{err}"),
                    _ => (),
                }
            }

            let now = std::time::Instant::now();
            if let Err(err) = terminal.draw(|frame| self.render(frame)) {
                log::error!("Error rendering frame: {err}");
            }
            let elapsed = now.elapsed().as_millis();
            log::info!("draw: {elapsed}ms {}fps", 1000 / elapsed);
        }

        tui::restore()?;

        Ok(())
    }

    fn render(&mut self, frame: &mut Frame) {
        let (display_width, display_height) = (64, 32);

        let [top_area, bottom_area] = Layout::new(
            Direction::Vertical,
            [
                Constraint::Length(display_height / 2 + 2),
                Constraint::Fill(1),
            ],
        )
        .split(frame.area())[..] else {
            unreachable!()
        };

        let [display_area, cpu_area] = Layout::new(
            Direction::Horizontal,
            [Constraint::Length(display_width + 2), Constraint::Fill(1)],
        )
        .split(top_area)[..] else {
            unreachable!()
        };

        let [disasm_area, mem_area, debug_area] = Layout::new(
            Direction::Horizontal,
            [
                Constraint::Length(37),
                Constraint::Length(62),
                Constraint::Fill(1),
            ],
        )
        .split(bottom_area)[..] else {
            unreachable!()
        };

        let [log_area, debugger_area] = Layout::new(
            Direction::Vertical,
            Constraint::from_ratios([(1, 3), (2, 3)]),
        )
        .split(debug_area)[..] else {
            unreachable!()
        };

        frame.render_widget(
            Block::new().title("CHIP-8").borders(Borders::ALL),
            display_area,
        );

        self.panels[0].render(frame, display_area, &self.state);
        self.panels[1].render(frame, cpu_area, &self.state);
        self.panels[2].render(frame, disasm_area, &self.state);
        self.panels[3].render(frame, mem_area, &self.state);
        self.panels[4].render(frame, log_area, &self.state);
        self.panels[5].render(frame, debugger_area, &self.state);
    }

    fn handle_key_event(&mut self, event: KeyEvent) {
        if let Some(focused) = self.panels.iter_mut().find(|p| p.has_focus()) {
            if focused.handle_key_event(event, &self.state) {
                return;
            }
        }

        match event.code {
            KeyCode::Char('1') => self.focus(0),
            KeyCode::Char('2') => self.focus(1),
            KeyCode::Char('3') => self.focus(2),
            KeyCode::Char('4') => self.focus(3),
            KeyCode::Char('5') => self.focus(4),
            KeyCode::Char('6') => self.focus(5),

            KeyCode::Char('q') => {
                self.cancellation_token.cancel();
                let _ = self.state.controller.send(EmulatorCommand::Stop);
            }

            KeyCode::Tab => self.focus_next(),
            KeyCode::Esc => self.unfocus(),
            _ => (),
        };
    }

    fn focus(&mut self, i: usize) {
        if !self.panels[i].has_focus() {
            self.unfocus();
        }
        self.panels[i].set_focus(true);
    }

    fn unfocus(&mut self) {
        if let Some(focused) = self.panels.iter_mut().find(|p| p.has_focus()) {
            focused.set_focus(false);
        }
    }

    fn focus_next(&mut self) {
        let panel_count = self.panels.len();
        if let Some((i, focused)) = self
            .panels
            .iter_mut()
            .enumerate()
            .find(|(_, p)| p.has_focus())
        {
            let next = (i + 1) % panel_count;
            focused.set_focus(false);
            self.panels[next].set_focus(true);
        } else {
            self.panels[0].set_focus(true);
        }
    }
}
