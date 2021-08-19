pub const WIDTH: usize = 64;
pub const HEIGHT: usize = 32;
pub const SCALE: usize = 10;
pub const SCREEN_WIDTH: usize = WIDTH * SCALE;
pub const SCREEN_HEIGHT: usize = HEIGHT * SCALE;

pub struct Display {
  pub screen: [u8; WIDTH * HEIGHT],
}

impl Display {
  pub fn new() -> Display {
    Display {
      screen: [0; WIDTH * HEIGHT],
    }
  }

  pub fn draw(&mut self, x: u8, y: u8, byte: u8) -> bool {
    let mut erased = false;
    let mut coord_x = x as usize;
    let mut coord_y = y as usize;
    let mut b = byte;

    for _ in 0..8 {
      coord_x %= WIDTH;
      coord_y %= HEIGHT;

      let index = coord_y * WIDTH + coord_x;
      let bit = b >> 7;
      let prev_value = self.screen[index];
      self.screen[index] ^= bit;

      if prev_value == 1 && self.screen[index] == 0 {
        erased = true;
      }
      coord_x += 1;
      b <<= 1;
    }
    return erased;
  }

  pub fn clear(&mut self) {
    self.screen = [0; WIDTH * HEIGHT];
  }
}
