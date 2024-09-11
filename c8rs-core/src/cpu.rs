use std::ops::{Index, IndexMut};

use crate::{
    display::Display, instructions::Register, memory::FONT_SPRITE_ADDR, Instruction, Memory,
};

pub type Registers = [u8; 16];

impl Index<Register> for Registers {
    type Output = u8;

    fn index(&self, index: Register) -> &Self::Output {
        &self[index as usize]
    }
}

impl IndexMut<Register> for Registers {
    fn index_mut(&mut self, index: Register) -> &mut Self::Output {
        &mut self[index as usize]
    }
}

#[derive(Debug)]
pub struct Cpu {
    pub registers: Registers,

    pub delay_timer: u8,
    pub sound_timer: u8,

    pub pc: u16,
    pub sp: u16,
    pub i: u16,

    pub(crate) mem: Memory,
    pub(crate) display: Display,
}

impl Cpu {
    pub(crate) fn new(mem: Memory, display: Display) -> Cpu {
        Cpu {
            registers: Default::default(),

            delay_timer: 0,
            sound_timer: 0,

            pc: 0x200,
            sp: 0x1FE,
            i: 0x000,

            mem,
            display,
        }
    }

    pub fn reset(&mut self) {
        self.pc = 0x200;
        self.sp = 0x1FE;
        self.display.clear();
    }

    pub fn step(&mut self) -> bool {
        let instr = Instruction::parse(self.mem.read_u16(self.pc));

        match self.execute(instr) {
            Some(pc) => {
                self.pc = pc;
                false
            }
            None => true,
        }
    }

    fn execute(&mut self, instr: Instruction) -> Option<u16> {
        match instr {
            Instruction::Cls => self.display.clear(),
            Instruction::Ret => self.pc = self.pop_stack(),
            Instruction::Jmp { addr } => {
                if addr == self.pc {
                    return None;
                }
                self.pc = addr;
            }
            Instruction::Call { addr } => {
                self.push_stack(self.pc);
                self.pc = addr;
            }
            Instruction::SkipEqImm { reg, byte } => {
                if self.registers[reg] == byte {
                    self.pc = self.pc.wrapping_add(2);
                }
            }
            Instruction::SkipNEqImm { reg, byte } => {
                if self.registers[reg] != byte {
                    self.pc = self.pc.wrapping_add(2);
                }
            }
            Instruction::SkipEqReg { regx, regy } => {
                if self.registers[regx] == self.registers[regy] {
                    self.pc = self.pc.wrapping_add(2);
                }
            }
            Instruction::LdImm { reg, byte } => self.registers[reg] = byte,
            Instruction::AddImm { reg, byte } => {
                self.registers[reg] = self.registers[reg].wrapping_add(byte)
            }
            Instruction::LdReg { regx, regy } => self.registers[regx] = self.registers[regy],
            Instruction::Or { regx, regy } => self.registers[regx] |= self.registers[regy],
            Instruction::And { regx, regy } => self.registers[regx] &= self.registers[regy],
            Instruction::Xor { regx, regy } => self.registers[regx] ^= self.registers[regy],
            Instruction::AddReg { regx, regy } => {
                let (val, carry) = self.registers[regx].overflowing_add(self.registers[regy]);
                self.registers[regx] = val;
                self.registers[Register::VF] = if carry { 1 } else { 0 };
            }
            Instruction::SubReg { regx, regy } => {
                self.registers[Register::VF] = if self.registers[regx] > self.registers[regy] {
                    1
                } else {
                    0
                };
                self.registers[regx] = self.registers[regx].wrapping_sub(self.registers[regy]);
            }
            Instruction::Shr { regx, regy } => {
                self.registers[Register::VF] = self.registers[regy] & 0x01;
                self.registers[regx] = self.registers[regy] >> 1;
            }
            Instruction::SubN { regx, regy } => {
                self.registers[Register::VF] = if self.registers[regy] > self.registers[regx] {
                    1
                } else {
                    0
                };
                self.registers[regx] = self.registers[regy].wrapping_sub(self.registers[regx]);
            }
            Instruction::Shl { regx, regy } => {
                self.registers[Register::VF] = self.registers[regy] & 0x80;
                self.registers[regx] = self.registers[regy] << 1;
            }
            Instruction::SkipNEqReg { regx, regy } => {
                if self.registers[regx] != self.registers[regy] {
                    self.pc = self.pc.saturating_add(2);
                }
            }
            Instruction::LdI { addr } => self.i = addr,
            Instruction::JmpReg { addr } => {
                self.pc = addr + self.registers[Register::V0] as u16;
            }
            // Instruction::Rnd { reg, byte } => todo!(),
            Instruction::Drw { regx, regy, len } => {
                let sprite = self.mem.read(self.i, len as u16);
                let collision =
                    self.display
                        .draw_sprite(self.registers[regx], self.registers[regy], sprite);
                self.registers[Register::VF] = collision as u8;
            }
            // Instruction::SkipPressed { reg } => todo!(),
            // Instruction::SkipNotPressed { reg } => todo!(),
            Instruction::LdDelayTimer { reg } => self.registers[reg] = self.delay_timer,
            // Instruction::LdKey { reg } => todo!(),
            Instruction::SetDelayTimer { reg } => self.delay_timer = self.registers[reg],
            Instruction::SetSoundTimer { reg } => self.sound_timer = self.registers[reg],
            Instruction::AddI { reg } => self.i = self.i.wrapping_add(self.registers[reg] as u16),
            Instruction::LdFont { reg } => {
                self.i = FONT_SPRITE_ADDR + self.registers[reg] as u16 * 5
            }
            Instruction::Bcd { reg } => {
                let val = self.registers[reg];
                self.mem.write_u8(self.i, val / 100);
                self.mem.write_u8(self.i + 1, (val / 10) % 10);
                self.mem.write_u8(self.i + 2, val % 10);
            }
            Instruction::StoreRegs { reg } => {
                for reg in 0..=reg as u16 {
                    self.mem
                        .write_u8(self.i + reg, self.registers[reg as usize]);
                }
            }
            Instruction::LoadRegs { reg } => {
                for reg in 0..=reg as u16 {
                    self.registers[reg as usize] = self.mem.read_u8(self.i + reg);
                }
            }
            _ => (),
        };

        match instr {
            Instruction::Jmp { .. } | Instruction::Call { .. } | Instruction::JmpReg { .. } => {
                Some(self.pc)
            }
            _ => Some(self.pc.wrapping_add(2)),
        }
    }

    fn push_stack(&mut self, addr: u16) {
        self.mem.write_u16(self.sp, addr);
        self.sp = self.sp.saturating_sub(2);
    }

    fn pop_stack(&mut self) -> u16 {
        self.sp = self.sp.saturating_add(2);
        self.mem.read_u16(self.sp)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{instructions::Register::*, Instruction::*};

    macro_rules! test_instr {
        ($instr:expr) => {{
            let mut cpu = Cpu::new(Memory::init(&[]), Display::default());

            let pc = cpu.execute($instr);

            (cpu, pc)
        }};
        ($instr:expr, DT => $value:expr) => {{
            let mut cpu = Cpu::new(Memory::init(&[]), Display::default());
            cpu.delay_timer = $value;

            let pc = cpu.execute($instr);

            (cpu, pc)
        }};
        ($instr:expr, I => $i_val:expr, $($register:expr => $value:expr),*) => {{
            let mut cpu = Cpu::new(Memory::init(&[]), Display::default());
            cpu.i = $i_val;
            $(
                cpu.registers[$register] = $value;
            )*

            let pc = cpu.execute($instr);

            (cpu, pc)
        }};
        ($instr:expr, $($register:expr => $value:expr),*) => {{
            let mut cpu = Cpu::new(Memory::init(&[]), Display::default());
            $(
                cpu.registers[$register] = $value;
            )*

            let pc = cpu.execute($instr);

            (cpu, pc)
        }};
    }

    #[test]
    fn test_cls() {
        let mut display = Display::default();
        display.draw_sprite(10, 10, &[0xF0, 0xA0, 0xBF]);

        let mut cpu = Cpu::new(Memory::init(&[]), display);
        cpu.execute(Cls);

        assert_eq!(cpu.display, Display::default());
    }

    #[test]
    fn test_ret() {
        let mut cpu = Cpu::new(Memory::init(&[]), Display::default());
        cpu.push_stack(0x2A8);

        let pc = cpu.execute(Ret);
        assert_eq!(pc, Some(0x2AA))
    }

    #[test]
    fn test_jmp() {
        let (_, pc) = test_instr!(Jmp { addr: 0x3FA });
        assert_eq!(pc, Some(0x3FA));
    }

    #[test]
    fn test_call() {
        let (mut cpu, pc) = test_instr!(Call { addr: 0x123 });
        assert_eq!(pc, Some(0x123));
        assert_eq!(cpu.pop_stack(), 0x200);
    }

    #[test]
    fn test_skip_eq_imm() {
        let (_, pc) = test_instr!(SkipEqImm { reg: VA, byte: 0xAB }, VA => 0xAB);
        assert_eq!(pc, Some(0x204));

        let (_, pc) = test_instr!(SkipEqImm { reg: VA, byte: 0xAF }, VA => 0xAB);
        assert_eq!(pc, Some(0x202));
    }

    #[test]
    fn test_skip_neq_imm() {
        let (_, pc) = test_instr!(SkipNEqImm { reg: VA, byte: 0xAF }, VA => 0xAB);
        assert_eq!(pc, Some(0x204));

        let (_, pc) = test_instr!(SkipNEqImm { reg: VA, byte: 0xAB }, VA => 0xAB);
        assert_eq!(pc, Some(0x202));
    }

    #[test]
    fn test_skip_eq_reg() {
        let (_, pc) = test_instr!(SkipEqReg { regx: V2, regy: V5 }, V2 => 0x21, V5 => 0x21);
        assert_eq!(pc, Some(0x204));

        let (_, pc) = test_instr!(SkipEqReg { regx: V2, regy: V5 }, V2 => 0x21, V5 => 0x22);
        assert_eq!(pc, Some(0x202));
    }

    #[test]
    fn test_ld_imm() {
        let (cpu, pc) = test_instr!(LdImm {
            reg: VD,
            byte: 0xFA
        });
        assert_eq!(cpu.registers[VD], 0xFA);
        assert_eq!(pc, Some(0x202));
    }

    #[test]
    fn test_add_imm() {
        let (cpu, pc) = test_instr!(AddImm { reg: V1, byte: 0x2 }, V1 => 0x7);
        assert_eq!(cpu.registers[V1], 0x9);
        assert_eq!(pc, Some(0x202));
    }

    #[test]
    fn test_ld_reg() {
        let (cpu, pc) = test_instr!(LdReg { regx: V3, regy: V4 }, V3 => 0x12, V4 => 0x34);
        assert_eq!(cpu.registers[V3], 0x34);
        assert_eq!(pc, Some(0x202));
    }

    #[test]
    fn test_or() {
        let (cpu, pc) = test_instr!(Or { regx: V0, regy: V1 }, V0 => 0b00101001, V1 => 0b11010010);
        assert_eq!(cpu.registers[V0], 0b11111011);
        assert_eq!(cpu.registers[V1], 0b11010010);
        assert_eq!(pc, Some(0x202));
    }

    #[test]
    fn test_and() {
        let (cpu, pc) = test_instr!(And { regx: V2, regy: V3 }, V2 => 0b01101101, V3 => 0b00101010);
        assert_eq!(cpu.registers[V2], 0b00101000);
        assert_eq!(cpu.registers[V3], 0b00101010);
        assert_eq!(pc, Some(0x202));
    }

    #[test]
    fn test_xor() {
        let (cpu, pc) = test_instr!(Xor { regx: VA, regy: VB }, VA => 0b11001010, VB => 0b10100100);
        assert_eq!(cpu.registers[VA], 0b01101110);
        assert_eq!(cpu.registers[VB], 0b10100100);
        assert_eq!(pc, Some(0x202));
    }

    #[test]
    fn test_add_reg() {
        let (cpu, pc) =
            test_instr!(AddReg { regx: V8, regy: V9 }, V8 => 0x40, V9 => 0x14, VF => 0x12);
        assert_eq!(cpu.registers[V8], 0x54);
        assert_eq!(cpu.registers[V9], 0x14);
        assert_eq!(cpu.registers[VF], 0x00);
        assert_eq!(pc, Some(0x202));

        let (cpu, pc) = test_instr!(AddReg { regx: V8, regy: V9 }, V8 => 0xF0, V9 => 0x10);
        assert_eq!(cpu.registers[V8], 0x00);
        assert_eq!(cpu.registers[V9], 0x10);
        assert_eq!(cpu.registers[VF], 0x01);
        assert_eq!(pc, Some(0x202));
    }

    #[test]
    fn test_sub_reg() {
        let (cpu, pc) = test_instr!(SubReg { regx: V1, regy: V2 }, V1 => 0x12, V2 => 0x0F);
        assert_eq!(cpu.registers[V1], 0x03);
        assert_eq!(cpu.registers[V2], 0x0F);
        assert_eq!(cpu.registers[VF], 0x01);
        assert_eq!(pc, Some(0x202));

        let (cpu, pc) = test_instr!(SubReg { regx: V1, regy: V2 }, V1 => 0x12, V2 => 0x1F);
        assert_eq!(cpu.registers[V1], 0xF3);
        assert_eq!(cpu.registers[V2], 0x1F);
        assert_eq!(cpu.registers[VF], 0x00);
        assert_eq!(pc, Some(0x202));
    }

    #[test]
    fn test_shift_right() {
        let (cpu, pc) = test_instr!(Shr { regx: V0, regy: V1 }, V1 => 0b00100101);
        assert_eq!(cpu.registers[V0], 0b00010010);
        assert_eq!(cpu.registers[V1], 0b00100101);
        assert_eq!(cpu.registers[VF], 0x01);
        assert_eq!(pc, Some(0x202));

        let (cpu, pc) = test_instr!(Shr { regx: V0, regy: V1 }, V1 => 0b00100100, VF => 0x12);
        assert_eq!(cpu.registers[V0], 0b00010010);
        assert_eq!(cpu.registers[V1], 0b00100100);
        assert_eq!(cpu.registers[VF], 0x00);
        assert_eq!(pc, Some(0x202));
    }

    #[test]
    fn test_sub_n() {
        let (cpu, pc) = test_instr!(SubN { regx: V1, regy: V2 }, V1 => 0x0F, V2 => 0x12);
        assert_eq!(cpu.registers[V1], 0x03);
        assert_eq!(cpu.registers[V2], 0x12);
        assert_eq!(cpu.registers[VF], 0x01);
        assert_eq!(pc, Some(0x202));

        let (cpu, pc) = test_instr!(SubN { regx: V1, regy: V2 }, V1 => 0x1F, V2 => 0x12);
        assert_eq!(cpu.registers[V1], 0xF3);
        assert_eq!(cpu.registers[V2], 0x12);
        assert_eq!(cpu.registers[VF], 0x00);
        assert_eq!(pc, Some(0x202));
    }

    #[test]
    fn test_ld_i() {
        let (cpu, pc) = test_instr!(LdI { addr: 0x2AB });
        assert_eq!(cpu.i, 0x2AB);
        assert_eq!(pc, Some(0x202));
    }

    #[test]
    fn test_jmp_reg() {
        let (_, pc) = test_instr!(JmpReg { addr: 0x342 }, V0 => 0x12);
        assert_eq!(pc, Some(0x354));
    }

    // TODO: test Rnd

    // TODO: test Drw

    // TODO: test SkipPressed

    // TODO: test SkipNotPressed

    #[test]
    fn test_ld_delay_timer() {
        let (cpu, pc) = test_instr!(LdDelayTimer { reg: V2 }, DT => 0x1F);
        assert_eq!(cpu.registers[V2], 0x1F);
        assert_eq!(pc, Some(0x202));
    }

    // TODO: test LdKey

    #[test]
    fn test_set_delay_timer() {
        let (cpu, pc) = test_instr!(SetDelayTimer { reg: V5 }, V5 => 0x12);
        assert_eq!(cpu.delay_timer, 0x12);
        assert_eq!(pc, Some(0x202));
    }

    #[test]
    fn test_set_sound_timer() {
        let (cpu, pc) = test_instr!(SetSoundTimer { reg: VA }, VA => 0x42);
        assert_eq!(cpu.sound_timer, 0x42);
        assert_eq!(pc, Some(0x202));
    }

    #[test]
    fn test_add_i() {
        let (cpu, pc) = test_instr!(AddI { reg: V0 }, I => 0x20F, V0 => 0x2);
        assert_eq!(cpu.i, 0x211);
        assert_eq!(pc, Some(0x202));
    }

    #[test]
    fn test_ld_font() {
        let (cpu, pc) = test_instr!(LdFont { reg: V1 }, V1 => 0x0A);
        assert_eq!(cpu.i, crate::memory::FONT_SPRITE_ADDR + (0x0A * 5));
        assert_eq!(pc, Some(0x202));
    }

    #[test]
    fn test_bcd() {
        let (cpu, pc) = test_instr!(Bcd { reg: VE }, I => 0x300, VE => 123);
        assert_eq!(cpu.mem.read_u8(0x300), 1);
        assert_eq!(cpu.mem.read_u8(0x301), 2);
        assert_eq!(cpu.mem.read_u8(0x302), 3);
        assert_eq!(pc, Some(0x202));
    }

    #[test]
    fn test_store_regs() {
        let (cpu, pc) = test_instr!(
            StoreRegs { reg: V5 },
            I => 0x300,
            V0 => 0x01, V1 => 0x23, V2 => 0x45, V3 => 0x67, V4 => 0x89, V5 => 0xAB
        );
        assert_eq!(cpu.mem.read(0x300, 6), [0x01, 0x23, 0x45, 0x67, 0x89, 0xAB]);
        assert_eq!(pc, Some(0x202));
    }

    #[test]
    fn test_load_regs() {
        let mem = Memory::init(&[0xAB, 0xCD, 0xEF]);
        let mut cpu = Cpu::new(mem, Display::default());
        cpu.i = 0x200;
        cpu.registers[V3] = 0x12;

        let pc = cpu.execute(LoadRegs { reg: V3 });

        assert_eq!(cpu.registers[V0], 0xAB);
        assert_eq!(cpu.registers[V1], 0xCD);
        assert_eq!(cpu.registers[V2], 0xEF);
        assert_eq!(cpu.registers[V3], 0x00);
        assert_eq!(pc, Some(0x202));
    }
}
