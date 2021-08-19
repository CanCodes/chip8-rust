extern crate minifb;

mod cpu;
mod display;
mod keyboard;
mod ram;

use cpu::Cpu;
use display::{SCREEN_HEIGHT, SCREEN_WIDTH};
use minifb::{Key, KeyRepeat, Window, WindowOptions};
use std::fs::File;
use std::io::Read;
use std::time::{Duration, Instant};

fn translate_keycode(key: Option<Key>) -> Option<u8> {
  match key {
    Some(Key::Key1) => Some(0x1),
    Some(Key::Key2) => Some(0x2),
    Some(Key::Key3) => Some(0x3),
    Some(Key::Key4) => Some(0xC),

    Some(Key::Q) => Some(0x4),
    Some(Key::W) => Some(0x5),
    Some(Key::E) => Some(0x6),
    Some(Key::R) => Some(0xD),

    Some(Key::A) => Some(0x7),
    Some(Key::S) => Some(0x8),
    Some(Key::D) => Some(0x9),
    Some(Key::F) => Some(0xE),

    Some(Key::Z) => Some(0xA),
    Some(Key::X) => Some(0x0),
    Some(Key::C) => Some(0xB),
    Some(Key::V) => Some(0xF),
    _ => None,
  }
}

fn main() {
  let mut file = File::open("c8games/KALEID").unwrap();
  let mut data: Vec<u8> = vec![];
  file.read_to_end(&mut data).unwrap();
  let mut cpu = Cpu::new();
  cpu.load_rom(&data);

  let mut window: Window = Window::new(
    "CHIP8",
    SCREEN_WIDTH,
    SCREEN_HEIGHT,
    WindowOptions::default(),
  )
  .unwrap();

  let mut buffer: Vec<u32> = vec![0; SCREEN_HEIGHT * SCREEN_WIDTH];

  let mut last_key_update_time = Instant::now();
  let mut last_instruction_run_time = Instant::now();
  let mut last_display_time = Instant::now();

  while window.is_open() && !window.is_key_pressed(Key::Escape, KeyRepeat::No) {
    let keys_pressed = window.get_keys_pressed(KeyRepeat::Yes);
    let key = match keys_pressed {
      Some(keys) => {
        if !keys.is_empty() {
          Some(keys[0])
        } else {
          None
        }
      }
      None => None,
    };

    let translated_key = translate_keycode(key);
    if translated_key.is_some()
      || Instant::now() - last_key_update_time >= Duration::from_millis(200)
    {
      last_key_update_time = Instant::now();
      cpu.keyboard.set_pressed_key(translated_key);
    }

    if Instant::now() - last_instruction_run_time > Duration::from_millis(2) {
      cpu.run_instruction();
      last_instruction_run_time = Instant::now();
    }

    if Instant::now() - last_display_time > Duration::from_millis(10) {
      let cpu_buffer = &cpu.display.screen;
      for y in 0..SCREEN_HEIGHT {
        let ycoord = y / 10;
        let offset = y * SCREEN_WIDTH;

        for x in 0..SCREEN_WIDTH {
          let index = ycoord * 64 + (x / 10);
          let pixel = cpu_buffer[index];
          let color = match pixel {
            0 => 0x000000,
            1 => 0xffffff,
            _ => unreachable!(),
          };
          buffer[offset + x] = color;
        }
      }
      window
        .update_with_buffer(&buffer, SCREEN_WIDTH, SCREEN_HEIGHT)
        .unwrap();
      last_display_time = Instant::now();
    }
  }
}
