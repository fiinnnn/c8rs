use std::{
    cell::UnsafeCell,
    collections::HashSet,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc,
    },
    thread,
    time::Duration,
};

pub use cpu::Cpu;
pub use debug::DebugCommand;
use display::Display;
pub use instructions::Instruction;
pub use memory::Memory;

pub mod cpu;
pub mod debug;
pub mod display;
pub mod instructions;
pub mod memory;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EmulatorCommand {
    Stop,
    DebugCommand(DebugCommand),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EmulatorState {
    Running,
    Paused,
    Halted,
}

pub struct Chip8Emulator {
    cmd_tx: Sender<EmulatorCommand>,
    inner: Arc<UnsafeCell<Chip8EmulatorInner>>,
}

impl Chip8Emulator {
    pub fn new(buf: &[u8]) -> Chip8Emulator {
        let (cmd_tx, cmd_rx) = channel();

        Chip8Emulator {
            cmd_tx,
            #[allow(clippy::arc_with_non_send_sync)]
            inner: Arc::new(UnsafeCell::new(Chip8EmulatorInner {
                ips: 10,
                state: EmulatorState::Paused,
                cpu: Cpu::new(Memory::init(buf), Display::default()),
                cmd_rx,
                breakpoints: HashSet::new(),
            })),
        }
    }

    pub fn controller(&self) -> EmulatorController {
        EmulatorController {
            cmd_tx: self.cmd_tx.clone(),
            emulator: self.inner.clone(),
        }
    }

    pub fn start(self) {
        let inner = unsafe { &mut *self.inner.get() };
        thread::spawn(move || {
            inner.run();
        });
    }
}

struct Chip8EmulatorInner {
    ips: u32,
    state: EmulatorState,
    cpu: Cpu,
    cmd_rx: Receiver<EmulatorCommand>,
    breakpoints: HashSet<u16>,
}

impl Chip8EmulatorInner {
    fn run(&mut self) {
        let mut interval = spin_sleep_util::interval(Duration::from_secs(1) / self.ips);

        loop {
            {
                let pc = self.cpu.pc;
                if self.breakpoints.contains(&pc) {
                    self.state = EmulatorState::Paused;
                    log::info!("Breakpoint hit: PC={pc:#06X}");
                }
            }

            if let Some(cmd) = match self.state {
                EmulatorState::Running => self.cmd_rx.try_recv().ok(),
                EmulatorState::Paused | EmulatorState::Halted => self.cmd_rx.recv().ok(),
            } {
                match cmd {
                    EmulatorCommand::Stop => break,
                    EmulatorCommand::DebugCommand(DebugCommand::IPS { ips }) => {
                        self.ips = ips;
                        interval = spin_sleep_util::interval(Duration::from_secs(1) / self.ips);
                        continue;
                    }
                    EmulatorCommand::DebugCommand(cmd) => {
                        if !self.handle_debug_cmd(cmd) {
                            continue;
                        }
                    }
                }
            }

            if self.cpu.step() {
                log::info!("CPU halted");
                self.state = EmulatorState::Halted;
            }

            interval.tick();
        }
    }

    fn handle_debug_cmd(&mut self, cmd: DebugCommand) -> bool {
        match cmd {
            DebugCommand::Step => true,
            DebugCommand::Pause => {
                self.state = EmulatorState::Paused;
                false
            }
            DebugCommand::Continue => {
                self.state = EmulatorState::Running;
                true
            }
            DebugCommand::Breakpoint { addr } => {
                if self.breakpoints.contains(&addr) {
                    self.breakpoints.remove(&addr);
                    log::info!("Breakpoint removed: {addr:#06X}");
                } else {
                    self.breakpoints.insert(addr);
                    log::info!("Breakpoint set: {addr:#06X}");
                }
                false
            }
            DebugCommand::Reset => {
                self.cpu.reset();
                false
            }
            DebugCommand::SetPc { addr } => {
                self.cpu.pc = addr;
                false
            }
            DebugCommand::IPS { .. } => false,
        }
    }
}

pub struct EmulatorController {
    cmd_tx: Sender<EmulatorCommand>,
    emulator: Arc<UnsafeCell<Chip8EmulatorInner>>,
}

impl EmulatorController {
    pub fn send(
        &self,
        cmd: EmulatorCommand,
    ) -> Result<(), std::sync::mpsc::SendError<EmulatorCommand>> {
        self.cmd_tx.send(cmd)
    }

    pub fn ips(&self) -> u32 {
        unsafe { &*self.emulator.get() }.ips
    }

    pub fn state(&self) -> EmulatorState {
        unsafe { &*self.emulator.get() }.state
    }

    pub fn cpu(&self) -> &Cpu {
        &unsafe { &*self.emulator.get() }.cpu
    }

    pub fn memory(&self) -> &Memory {
        &unsafe { &*self.emulator.get() }.cpu.mem
    }

    pub fn display(&self) -> &Display {
        &unsafe { &*self.emulator.get() }.cpu.display
    }

    pub fn breakpoints(&self) -> &HashSet<u16> {
        &unsafe { &*self.emulator.get() }.breakpoints
    }
}
