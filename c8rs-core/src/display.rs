use bitvec::array::BitArray;

const DISPLAY_WIDTH: usize = 64;
const DISPLAY_HEIGHT: usize = 32;

#[derive(Default, Debug, PartialEq)]
pub struct Display {
    buffer: BitArray<[usize; DISPLAY_HEIGHT]>,
}

impl Display {
    pub(crate) fn clear(&mut self) {
        self.buffer = BitArray::new([0; DISPLAY_HEIGHT]);
    }

    pub(crate) fn draw_sprite(&mut self, x: u8, y: u8, sprite: &[u8]) -> bool {
        let mut collision = false;

        for (row, byte) in sprite.iter().enumerate() {
            let py = (y as usize + row) % DISPLAY_HEIGHT;

            for col in 0..8 {
                let px = (x as usize + col) % DISPLAY_WIDTH;
                let bit = byte & (1 << (7 - col)) != 0;
                let i = py * DISPLAY_WIDTH + px;
                collision |= self.set_pixel(i, bit);
            }
        }

        collision
    }

    fn set_pixel(&mut self, i: usize, bit: bool) -> bool {
        let Some(mut pixel) = self.buffer.get_mut(i) else {
            return false;
        };

        let prev = *pixel;
        let new = prev ^ bit;
        *pixel = new;

        prev && !new
    }

    pub fn get_dimensions(&self) -> (usize, usize) {
        (DISPLAY_WIDTH, DISPLAY_HEIGHT)
    }

    pub fn get_pixels(&self) -> Vec<bool> {
        self.buffer.iter().map(|b| *b).collect()
    }
}
