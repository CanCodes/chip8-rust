use crate::display::Display;
use crate::keyboard::Keyboard;
use crate::ram::Ram;
use std::time;

use rand;
use rand::{thread_rng, Rng};

const START_FROM: u16 = 0x200;

pub struct Cpu {
  pub ram: Ram,
  pub display: Display,
  pub keyboard: Keyboard,
  vx: [u8; 16],
  pc: u16,
  i: u16,
  stack: Vec<u16>,
  rng: rand::rngs::ThreadRng,
  dt: u8,
  st: u8,
  dts: time::Instant,
}

impl Cpu {
  pub fn new() -> Cpu {
    Cpu {
      ram: Ram::new(),
      display: Display::new(),
      keyboard: Keyboard::new(),
      vx: [0; 16],
      pc: START_FROM,
      i: 0,
      stack: vec![],
      rng: thread_rng(),
      dt: 0,
      st: 0,
      dts: time::Instant::now(),
    }
  }

  pub fn load_rom(&mut self, data: &Vec<u8>) {
    for i in 0..data.len() {
      self.ram.write_byte((START_FROM + i as u16) as u16, data[i]);
    }
  }

  pub fn run_instruction(&mut self) {
    let h = self.ram.read_byte(self.pc) as u16;
    let l = self.ram.read_byte(self.pc + 1) as u16;

    let inst: u16 = (h << 8) | l;

    let nnn = inst & 0x0FFF;
    let nn = (inst & 0x0FF) as u8;
    let n = (inst & 0x00F) as u8;
    let x = ((inst & 0x0F00) >> 8) as u8;
    let y = ((inst & 0x00F0) >> 4) as u8;

    match (inst & 0xF000) >> 12 {
      0x0 => match nn {
        0xE0 => {
          self.display.clear();
          self.pc += 2;
        }
        0xEE => {
          let addr = self.stack.pop().unwrap();
          self.pc = addr;
        }
        _ => panic!("IDK WHAT THIS IS"),
      },
      0x1 => {
        //SET PC TO NNN
        self.pc = nnn;
      }
      0x2 => {
        self.stack.push(self.pc + 2);
        self.pc = nnn;
      }
      0x3 => {
        let vx = self.vx[x as usize];
        self.pc += if vx == nn { 4 } else { 2 };
      }
      0x4 => {
        let vx = self.vx[x as usize];
        self.pc += if vx != nn { 4 } else { 2 };
      }
      0x5 => {
        let vx = self.vx[x as usize];
        let vy = self.vx[y as usize];
        self.pc += if vx == vy { 4 } else { 2 };
      }
      0x6 => {
        self.vx[x as usize] = nn;
        self.pc += 2;
      }
      0x7 => {
        let a = self.vx[x as usize];
        self.vx[x as usize] = a.wrapping_add(nn);
        self.pc += 2;
      }
      0x8 => {
        let vx = self.vx[x as usize];
        let vy = self.vx[y as usize];

        match n {
          0x0 => {
            self.vx[x as usize] = vy;
          }
          0x1 => {
            self.vx[x as usize] = vx | vy;
          }
          0x2 => {
            self.vx[x as usize] = vx & vy;
          }
          0x3 => {
            self.vx[x as usize] = vx ^ vy;
          }
          0x4 => {
            let s: u16 = vx as u16 + vy as u16;
            self.vx[x as usize] = s as u8;
            self.vx[0xF as usize] = if s > 0xFF { 1 } else { 0 }
          }
          0x5 => {
            let d: i8 = vx as i8 - vy as i8;
            self.vx[x as usize] = d as u8;
            self.vx[0xF as usize] = if d < 0 { 1 } else { 0 }
          }
          0x6 => {
            self.vx[0xF as usize] = vx & 0x1;
            self.vx[x as usize] = vx >> 1;
          }
          0x7 => {
            let d: i8 = vy as i8 - vx as i8;
            self.vx[x as usize] = d as u8;
            self.vx[0xF as usize] = if d < 0 { 1 } else { 0 }
          }
          0xE => {
            self.vx[x as usize] = vx << 1;
            self.vx[0xF as usize] = (vx & 0x80) >> 7;
          }
          _ => {
            panic!("Don't know what to do with this instruction: {:#X}", inst)
          }
        }
        self.pc += 2;
      }
      0x9 => {
        let vx = self.vx[x as usize];
        let vy = self.vx[y as usize];
        self.pc += if vx != vy { 4 } else { 2 }
      }
      0xA => {
        self.i = nnn;
        self.pc += 2;
      }
      0xB => {
        self.pc = self.vx[0usize] as u16 + nnn;
      }
      0xC => {
        let num: u8 = self.rng.gen_range(0..255);
        self.vx[x as usize] = num & nn;
        self.pc += 2;
      }
      0xD => {
        let vx = self.vx[x as usize];
        let vy = self.vx[y as usize];
        self.draw(vx, vy, n);
        self.pc += 2;
      }
      0xE => match nn {
        0xA1 => {
          let key = self.vx[x as usize];

          self.pc += if self.keyboard.pressed_key != Some(key) {
            4
          } else {
            2
          }
        }
        0x9E => {
          let key = self.vx[x as usize];
          self.pc += if self.keyboard.pressed_key == Some(key) {
            4
          } else {
            2
          }
        }
        _ => panic!("Don't know what to do with this instruction: {:#X}", inst),
      },
      0xF => match nn {
        0x1E => {
          let vx = self.vx[x as usize];
          self.i += vx as u16;
          self.pc += 2;
        }
        0x18 => {
          self.st = self.vx[x as usize];
          self.pc += 2;
        }
        0x07 => {
          self.vx[x as usize] = self.get_timer();
          self.pc += 2;
        }
        0x0A => {
          if let Some(val) = self.keyboard.pressed_key {
            self.vx[x as usize] = val;
            self.pc += 2;
          }
        }
        0x15 => {
          self.set_timer(self.vx[x as usize]);
          self.pc += 2;
        }
        0x29 => {
          self.i = self.vx[x as usize] as u16 * 5;
          self.pc += 2;
        }
        0x33 => {
          let vx = self.vx[x as usize];
          self.ram.write_byte(self.i, vx / 100);
          self.ram.write_byte(self.i + 1, (vx % 100) / 10);
          self.ram.write_byte(self.i + 2, vx % 10);
          self.pc += 2;
        }
        0x55 => {
          for index in 0..(x + 1) {
            let val = self.vx[index as usize];
            self.ram.write_byte(self.i + index as u16, val);
          }
          self.i += x as u16 + 1;
          self.pc += 2;
        }
        0x65 => {
          for index in 0..x + 1 {
            let val = self.vx[index as usize];
            self.vx[index as usize] = val;
          }
          self.i += x as u16 + 1;
          self.pc += 2;
        }
        _ => panic!("Don't know what to do with this instruction: {:#X}", inst),
      },
      _ => panic!("Don't know what to do with this instruction: {:#X}", inst),
    }
  }

  pub fn draw(&mut self, x: u8, y: u8, n: u8) {
    let mut vf_state = false;
    for sprite_y in 0..n {
      let d = self.ram.read_byte(self.i + sprite_y as u16);
      if self.display.draw(x, y + sprite_y, d) {
        vf_state = true;
      }
    }
    self.vx[0xF as usize] = if vf_state { 1 } else { 0 };
  }

  pub fn get_timer(&self) -> u8 {
    let diff = time::Instant::now() - self.dts;
    let ms = diff.as_millis();
    let ticks = ms / 16;
    if ticks >= self.dt as u128 {
      0
    } else {
      self.dt - ticks as u8
    }
  }

  pub fn set_timer(&mut self, value: u8) {
    self.dts = time::Instant::now();
    self.dt = value;
  }
}
