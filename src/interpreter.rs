
pub struct Context {
  tape: Vec<isize>,
  current_index: usize,
}

impl Context {
  pub fn new() -> Self {
    Context {
      tape: vec![0; 1],
      current_index: 0,
    }
  }

  pub fn move_right(&mut self) {
    self.current_index += 1;
    while self.current_index >= self.tape.len() {
      self.tape.push(0);
    }
  }

  pub fn move_left(&mut self) {
    if self.current_index > 0 {
      self.current_index -= 1;
    }
  }

  pub fn write(&mut self, value: isize) {
    self.tape[self.current_index] = value;
  }

  pub fn read(&self) -> isize {
    self.tape[self.current_index]
  }

  pub fn increment(&mut self) {
    self.tape[self.current_index] += 1;
  }

  pub fn decrement(&mut self) {
    self.tape[self.current_index] -= 1;
  }

  pub fn current_cell_is_zero(&mut self) -> bool {
    self.tape[self.current_index] == 0
  }
}
