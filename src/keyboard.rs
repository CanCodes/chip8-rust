pub struct Keyboard {
  pub pressed_key: Option<u8>,
}

impl Keyboard {
  pub fn new() -> Keyboard {
    Keyboard { pressed_key: None }
  }

  pub fn set_pressed_key(&mut self, key: Option<u8>) {
    self.pressed_key = key;
  }
}
