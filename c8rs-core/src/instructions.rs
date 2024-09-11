macro_rules! byte {
    ($n0:expr, $n1:expr) => {
        ($n0 << 4) | $n1
    };
}

macro_rules! addr {
    ($n0:expr, $n1: expr, $n2: expr) => {
        (($n0 as u16) << 8) | (($n1 as u16) << 4) | $n2 as u16
    };
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Register {
    V0 = 0x0,
    V1 = 0x1,
    V2 = 0x2,
    V3 = 0x3,
    V4 = 0x4,
    V5 = 0x5,
    V6 = 0x6,
    V7 = 0x7,
    V8 = 0x8,
    V9 = 0x9,
    VA = 0xA,
    VB = 0xB,
    VC = 0xC,
    VD = 0xD,
    VE = 0xE,
    VF = 0xF,
}

impl std::fmt::Display for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Register::V0 => "V0",
                Register::V1 => "V1",
                Register::V2 => "V2",
                Register::V3 => "V3",
                Register::V4 => "V4",
                Register::V5 => "V5",
                Register::V6 => "V6",
                Register::V7 => "V7",
                Register::V8 => "V8",
                Register::V9 => "V9",
                Register::VA => "VA",
                Register::VB => "VB",
                Register::VC => "VC",
                Register::VD => "VD",
                Register::VE => "VE",
                Register::VF => "VF",
            }
        )
    }
}

impl From<u8> for Register {
    fn from(value: u8) -> Self {
        match value {
            0x0 => Register::V0,
            0x1 => Register::V1,
            0x2 => Register::V2,
            0x3 => Register::V3,
            0x4 => Register::V4,
            0x5 => Register::V5,
            0x6 => Register::V6,
            0x7 => Register::V7,
            0x8 => Register::V8,
            0x9 => Register::V9,
            0xA => Register::VA,
            0xB => Register::VB,
            0xC => Register::VC,
            0xD => Register::VD,
            0xE => Register::VE,
            0xF => Register::VF,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Instruction {
    /// 00E0
    /// Clear screen
    Cls,

    /// 00EE
    /// Return from subroutine
    Ret,

    /// 1nnn
    /// Jump to addr `nnn`
    Jmp {
        addr: u16,
    },

    /// 2nnn
    /// Call subroutine at `nnn`
    Call {
        addr: u16,
    },

    /// 3xkk
    /// Skip next instruction if `Vx == kk`
    SkipEqImm {
        reg: Register,
        byte: u8,
    },

    /// 4xkk
    /// Skip next instruction if `Vx != kk`
    SkipNEqImm {
        reg: Register,
        byte: u8,
    },

    /// 5xy0
    /// Skip next instruction if `Vx == Vy`
    SkipEqReg {
        regx: Register,
        regy: Register,
    },

    /// 6xkk
    /// Load byte kk into register x
    LdImm {
        reg: Register,
        byte: u8,
    },

    /// 7xkk
    /// Add byte kk to register x
    AddImm {
        reg: Register,
        byte: u8,
    },

    /// 8xy0
    /// Store value of register y in register xy0
    LdReg {
        regx: Register,
        regy: Register,
    },

    /// 8xy1
    /// Bitwise OR `Vx = Vx | Vy`
    Or {
        regx: Register,
        regy: Register,
    },

    /// 8xy2
    /// Bitwise AND `Vx = Vx & Vy`
    And {
        regx: Register,
        regy: Register,
    },

    /// 8xy3
    /// Bitwise XOR `Vx = Vx ^ Vy`
    Xor {
        regx: Register,
        regy: Register,
    },

    /// 8xy4
    /// Add register values: `Vx = Vx + Vy`
    /// Set carry flag VF.
    AddReg {
        regx: Register,
        regy: Register,
    },

    /// 8xy5
    /// Subtract register values: `Vx = Vx - Vy`
    /// Set VF if `Vx > Vy`
    SubReg {
        regx: Register,
        regy: Register,
    },

    /// 8xy6
    /// Shift right: `Vx = Vy >> 1`
    /// Set VF to LSB before shift
    Shr {
        regx: Register,
        regy: Register,
    },

    /// 8xy7
    /// Subtract register values: `Vx = Vy - Vx`
    /// Set VF if `Vy > Vx`
    SubN {
        regx: Register,
        regy: Register,
    },

    /// 8xyE
    /// Shift left: `Vx = Vy << 1`
    /// Set VF to MSB before shift
    Shl {
        regx: Register,
        regy: Register,
    },

    /// 9xy0
    /// Skip next instruction if `Vx != Vy`
    SkipNEqReg {
        regx: Register,
        regy: Register,
    },

    /// Annn
    /// Set value of register I to `nnn`
    LdI {
        addr: u16,
    },

    /// Bnnn
    /// Jump to location `nnn + V0`
    JmpReg {
        addr: u16,
    },

    /// Cxkk
    /// Set `Vx = random byte & kk`
    Rnd {
        reg: Register,
        byte: u8,
    },

    /// Dxyn
    /// Draw n-byte sprite from location I at `(Vx, Vy)`, set VF if collision
    Drw {
        regx: Register,
        regy: Register,
        len: u8,
    },

    /// Ex9E
    /// Skip next instruction if key `Vx` is pressed
    SkipPressed {
        reg: Register,
    },

    /// ExA1
    /// Skip next instruction if key `x` is not pressed
    SkipNotPressed {
        reg: Register,
    },

    /// Fx07
    /// Set `Vx` to delay timer value
    LdDelayTimer {
        reg: Register,
    },

    /// Fx0A
    /// Wait for key press, store value in `Vx`
    LdKey {
        reg: Register,
    },

    /// Fx15
    /// Set delay timer to `Vx`
    SetDelayTimer {
        reg: Register,
    },

    /// Fx18
    /// Set sound timer to `Vx`
    SetSoundTimer {
        reg: Register,
    },

    /// Fx1E
    /// Add `Vx` to `I`
    AddI {
        reg: Register,
    },

    /// Fx29
    /// Set I to sprite for digit `Vx`
    LdFont {
        reg: Register,
    },

    /// Fx33
    /// Store binary-coded decimal representation of `Vx` in
    /// `I`, `I + 1` and `I + 2`
    Bcd {
        reg: Register,
    },

    /// Fx55
    /// Store registers `V0` through `Vx` in memory starting at location `I`
    StoreRegs {
        reg: Register,
    },

    /// Fx65
    /// Read registerx `V0` through `Vx` from memory starting at location `I`
    LoadRegs {
        reg: Register,
    },

    Unknown(u16),
}

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Instruction::Cls => write!(f, "CLS"),
            Instruction::Ret => write!(f, "RET"),
            Instruction::Jmp { addr } => write!(f, "JMP {addr:#06X}"),
            Instruction::Call { addr } => write!(f, "CALL {addr:#06X}"),
            Instruction::SkipEqImm { reg, byte } => write!(f, "SE {reg}, {byte:#04X}"),
            Instruction::SkipNEqImm { reg, byte } => write!(f, "SNE {reg}, {byte:#04X}"),
            Instruction::SkipEqReg { regx, regy } => write!(f, "SE {regx}, {regy}"),
            Instruction::LdImm { reg, byte } => write!(f, "LD {reg}, {byte:#04X}"),
            Instruction::AddImm { reg, byte } => write!(f, "ADD {reg}, {byte:#04X}"),
            Instruction::LdReg { regx, regy } => write!(f, "LD {regx}, {regy}"),
            Instruction::Or { regx, regy } => write!(f, "OR {regx}, {regy}"),
            Instruction::And { regx, regy } => write!(f, "AND {regx}, {regy}"),
            Instruction::Xor { regx, regy } => write!(f, "XOR {regx}, {regy}"),
            Instruction::AddReg { regx, regy } => write!(f, "ADD {regx}, {regy}"),
            Instruction::SubReg { regx, regy } => write!(f, "SUB {regx}, {regy}"),
            Instruction::Shr { regx, regy } => write!(f, "SHR {regx}, {regy}"),
            Instruction::SubN { regx, regy } => write!(f, "SUBN {regx}, {regy}"),
            Instruction::Shl { regx, regy } => write!(f, "SHL {regx}, {regy}"),
            Instruction::SkipNEqReg { regx, regy } => write!(f, "SNE {regx}, {regy}"),
            Instruction::LdI { addr } => write!(f, "LD I, {addr:#06X}"),
            Instruction::JmpReg { addr } => write!(f, "JMP V0, {addr:#06X}"),
            Instruction::Rnd { reg, byte } => write!(f, "RND {reg}, {byte:#04X}"),
            Instruction::Drw { regx, regy, len } => write!(f, "DRW {regx}, {regy}, {len:#04X}"),
            Instruction::SkipPressed { reg } => write!(f, "SKP {reg}"),
            Instruction::SkipNotPressed { reg } => write!(f, "SKNP {reg}"),
            Instruction::LdDelayTimer { reg } => write!(f, "LD {reg}, DT"),
            Instruction::LdKey { reg } => write!(f, "LD {reg}, K"),
            Instruction::SetDelayTimer { reg } => write!(f, "LD DT, {reg}"),
            Instruction::SetSoundTimer { reg } => write!(f, "LD ST, {reg}"),
            Instruction::AddI { reg } => write!(f, "ADD I, {reg}"),
            Instruction::LdFont { reg } => write!(f, "LD F, {reg}"),
            Instruction::Bcd { reg } => write!(f, "BCD {reg}"),
            Instruction::StoreRegs { reg } => write!(f, "LD [I], {reg}"),
            Instruction::LoadRegs { reg } => write!(f, "LD {reg}, [I]"),
            Instruction::Unknown(op) => write!(f, "unknown ({op:#06X})"),
        }
    }
}

impl Instruction {
    pub fn parse(op: u16) -> Instruction {
        let op0 = ((op & 0xF000) >> 12) as u8;
        let op1 = ((op & 0x0F00) >> 8) as u8;
        let op2 = ((op & 0x00F0) >> 4) as u8;
        let op3 = (op & 0x000F) as u8;

        match (op0, op1, op2, op3) {
            (0x0, 0x0, 0xE, 0x0) => Instruction::Cls,
            (0x0, 0x0, 0xE, 0xE) => Instruction::Ret,
            (0x1, n0, n1, n2) => Instruction::Jmp {
                addr: addr!(n0, n1, n2),
            },
            (0x2, n0, n1, n2) => Instruction::Call {
                addr: addr!(n0, n1, n2),
            },
            (0x3, x, n0, n1) => Instruction::SkipEqImm {
                reg: x.into(),
                byte: byte!(n0, n1),
            },
            (0x4, x, n0, n1) => Instruction::SkipNEqImm {
                reg: x.into(),
                byte: byte!(n0, n1),
            },
            (0x5, x, y, 0) => Instruction::SkipEqReg {
                regx: x.into(),
                regy: y.into(),
            },
            (0x6, x, n0, n1) => Instruction::LdImm {
                reg: x.into(),
                byte: byte!(n0, n1),
            },
            (0x7, x, n0, n1) => Instruction::AddImm {
                reg: x.into(),
                byte: byte!(n0, n1),
            },
            (0x8, x, y, 0x0) => Instruction::LdReg {
                regx: x.into(),
                regy: y.into(),
            },
            (0x8, x, y, 0x1) => Instruction::Or {
                regx: x.into(),
                regy: y.into(),
            },
            (0x8, x, y, 0x2) => Instruction::And {
                regx: x.into(),
                regy: y.into(),
            },
            (0x8, x, y, 0x3) => Instruction::Xor {
                regx: x.into(),
                regy: y.into(),
            },
            (0x8, x, y, 0x4) => Instruction::AddReg {
                regx: x.into(),
                regy: y.into(),
            },
            (0x8, x, y, 0x5) => Instruction::SubReg {
                regx: x.into(),
                regy: y.into(),
            },
            (0x8, x, y, 0x6) => Instruction::Shr {
                regx: x.into(),
                regy: y.into(),
            },
            (0x8, x, y, 0x7) => Instruction::SubN {
                regx: x.into(),
                regy: y.into(),
            },
            (0x8, x, y, 0xE) => Instruction::Shl {
                regx: x.into(),
                regy: y.into(),
            },
            (0x9, x, y, 0x0) => Instruction::SkipNEqReg {
                regx: x.into(),
                regy: y.into(),
            },
            (0xA, n0, n1, n2) => Instruction::LdI {
                addr: addr!(n0, n1, n2),
            },
            (0xB, n0, n1, n2) => Instruction::JmpReg {
                addr: addr!(n0, n1, n2),
            },
            (0xC, x, n0, n1) => Instruction::Rnd {
                reg: x.into(),
                byte: byte!(n0, n1),
            },
            (0xD, x, y, n) => Instruction::Drw {
                regx: x.into(),
                regy: y.into(),
                len: n,
            },
            (0xE, x, 0x9, 0xE) => Instruction::SkipPressed { reg: x.into() },
            (0xE, x, 0xA, 0x1) => Instruction::SkipNotPressed { reg: x.into() },
            (0xF, x, 0x0, 0x7) => Instruction::LdDelayTimer { reg: x.into() },
            (0xF, x, 0x0, 0xA) => Instruction::LdKey { reg: x.into() },
            (0xF, x, 0x1, 0x5) => Instruction::SetDelayTimer { reg: x.into() },
            (0xF, x, 0x1, 0x8) => Instruction::SetSoundTimer { reg: x.into() },
            (0xF, x, 0x1, 0xE) => Instruction::AddI { reg: x.into() },
            (0xF, x, 0x2, 0x9) => Instruction::LdFont { reg: x.into() },
            (0xF, x, 0x3, 0x3) => Instruction::Bcd { reg: x.into() },
            (0xF, x, 0x5, 0x5) => Instruction::StoreRegs { reg: x.into() },
            (0xF, x, 0x6, 0x5) => Instruction::LoadRegs { reg: x.into() },
            _ => Instruction::Unknown(op),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_instruction() {
        let tests = [
            (0x00E0, Instruction::Cls),
            (0x00EE, Instruction::Ret),
            (0x1123, Instruction::Jmp { addr: 0x123 }),
            (0x2123, Instruction::Call { addr: 0x123 }),
            (
                0x3123,
                Instruction::SkipEqImm {
                    reg: Register::V1,
                    byte: 0x23,
                },
            ),
            (
                0x4E23,
                Instruction::SkipNEqImm {
                    reg: Register::VE,
                    byte: 0x23,
                },
            ),
            (
                0x53A0,
                Instruction::SkipEqReg {
                    regx: Register::V3,
                    regy: Register::VA,
                },
            ),
            (
                0x6739,
                Instruction::LdImm {
                    reg: Register::V7,
                    byte: 0x39,
                },
            ),
            (
                0x7D94,
                Instruction::AddImm {
                    reg: Register::VD,
                    byte: 0x94,
                },
            ),
            (
                0x8120,
                Instruction::LdReg {
                    regx: Register::V1,
                    regy: Register::V2,
                },
            ),
            (
                0x8121,
                Instruction::Or {
                    regx: Register::V1,
                    regy: Register::V2,
                },
            ),
            (
                0x8122,
                Instruction::And {
                    regx: Register::V1,
                    regy: Register::V2,
                },
            ),
            (
                0x8123,
                Instruction::Xor {
                    regx: Register::V1,
                    regy: Register::V2,
                },
            ),
            (
                0x8124,
                Instruction::AddReg {
                    regx: Register::V1,
                    regy: Register::V2,
                },
            ),
            (
                0x8125,
                Instruction::SubReg {
                    regx: Register::V1,
                    regy: Register::V2,
                },
            ),
            (
                0x8126,
                Instruction::Shr {
                    regx: Register::V1,
                    regy: Register::V2,
                },
            ),
            (
                0x8127,
                Instruction::SubN {
                    regx: Register::V1,
                    regy: Register::V2,
                },
            ),
            (
                0x812E,
                Instruction::Shl {
                    regx: Register::V1,
                    regy: Register::V2,
                },
            ),
            (
                0x98F0,
                Instruction::SkipNEqReg {
                    regx: Register::V8,
                    regy: Register::VF,
                },
            ),
            (0xA123, Instruction::LdI { addr: 0x123 }),
            (0xB123, Instruction::JmpReg { addr: 0x123 }),
            (
                0xCB12,
                Instruction::Rnd {
                    reg: Register::VB,
                    byte: 0x12,
                },
            ),
            (
                0xDE51,
                Instruction::Drw {
                    regx: Register::VE,
                    regy: Register::V5,
                    len: 0x1,
                },
            ),
            (0xE29E, Instruction::SkipPressed { reg: Register::V2 }),
            (0xE5A1, Instruction::SkipNotPressed { reg: Register::V5 }),
            (0xF107, Instruction::LdDelayTimer { reg: Register::V1 }),
            (0xF10A, Instruction::LdKey { reg: Register::V1 }),
            (0xF115, Instruction::SetDelayTimer { reg: Register::V1 }),
            (0xF118, Instruction::SetSoundTimer { reg: Register::V1 }),
            (0xF11E, Instruction::AddI { reg: Register::V1 }),
            (0xF129, Instruction::LdFont { reg: Register::V1 }),
            (0xF133, Instruction::Bcd { reg: Register::V1 }),
            (0xF155, Instruction::StoreRegs { reg: Register::V1 }),
            (0xF165, Instruction::LoadRegs { reg: Register::V1 }),
        ];

        for (op, i) in tests {
            assert_eq!(Instruction::parse(op), i)
        }
    }
}
