pub const MEM_SIZE: usize = 4096;

pub const FONT_SPRITE_ADDR: u16 = 0x100;
const FONT_SPRITES: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

#[derive(Debug)]
pub struct Memory {
    bytes: [u8; MEM_SIZE],
}

impl Default for Memory {
    fn default() -> Self {
        Self { bytes: [0; 4096] }
    }
}

impl Memory {
    pub fn init(buf: &[u8]) -> Memory {
        let mut m = Memory::default();
        m.write(0x200, buf);
        m.write(FONT_SPRITE_ADDR, &FONT_SPRITES);
        m
    }

    pub fn read_u8(&self, addr: u16) -> u8 {
        self.bytes[addr as usize]
    }

    pub fn write_u8(&mut self, addr: u16, val: u8) {
        self.bytes[addr as usize] = val;
    }

    pub fn read_u16(&self, addr: u16) -> u16 {
        ((self.bytes[addr as usize] as u16) << 8) | self.bytes[(addr + 1) as usize] as u16
    }

    pub fn write_u16(&mut self, addr: u16, val: u16) {
        self.write_u8(addr, (val >> 8) as u8);
        self.write_u8(addr + 1, val as u8);
    }

    pub fn read(&self, addr: u16, len: u16) -> &[u8] {
        let addr = addr as usize;
        &self.bytes[addr..addr + len as usize]
    }

    pub fn write(&mut self, addr: u16, data: &[u8]) {
        let addr = addr as usize;
        self.bytes[addr..addr + data.len()].copy_from_slice(data);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rw_u8() {
        let mut m = Memory::default();

        m.write_u8(0x200, 0xAB);

        assert_eq!(m.read_u8(0x200), 0xAB);
    }

    #[test]
    fn test_rw_u16() {
        let mut m = Memory::default();

        m.write_u16(0x200, 0x1234);

        assert_eq!(m.read_u16(0x200), 0x1234);
    }

    #[test]
    fn test_rw() {
        let mut m = Memory::default();

        let data = [0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xDE];

        m.write(0x200, &data);

        assert_eq!(m.read(0x200, 8), data);
    }
}
