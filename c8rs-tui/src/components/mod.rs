use ratatui::{crossterm::event::KeyEvent, layout::Rect, Frame};

mod cpu;
mod debug;
mod disasm;
mod display;
mod log;
mod mem;

pub use cpu::CpuComponent;
pub use debug::DebuggerComponent;
pub use disasm::DisassemblyComponent;
pub use display::DisplayComponent;
pub use log::LogComponent;
pub use mem::MemoryComponent;

use crate::app::AppState;

pub trait Component {
    fn handle_key_event(&mut self, event: KeyEvent, state: &AppState) -> bool;

    fn render(&mut self, f: &mut Frame<'_>, area: Rect, state: &AppState);

    fn has_focus(&self) -> bool;

    fn set_focus(&mut self, focus: bool);
}
