
use std;
use std::io::{Write};
use std::sync::{Arc, Barrier};
use std::sync::mpsc::Sender;
use std::collections::VecDeque;
use parse::EOF;
use parse::Code;

pub struct Repl {
  // TODO: Better name
  tx: Sender<Vec<u8>>,
  barrier: Arc<Barrier>,
  running: bool,
}

impl Repl {
  pub fn new() -> Self {
    let barrier = Arc::new(Barrier::new(2));
    let (tx, rx) = std::sync::mpsc::channel();
    let mut code = Code::new_from_channel(rx, barrier.clone());
    std::thread::spawn(move|| {
      code.parse_and_run();
    });
    Repl {
      tx: tx,
      barrier: barrier,
      running: false,
    }
  }

  pub fn start(&mut self) {
    self.running = true;
    Repl::display_carrot(false);
    self.barrier.wait();
    while self.running {
      let mut input = String::new();
      match Repl::read_line(&mut input) {
        Ok(num_read) => {
          if num_read == 0 {
            self.send(vec![EOF]);
            self.running = false;
          }
          else if num_read == 1 {
            continue;
          }
          else {
            let will_output = self.interpret_command(&input);
            if self.running {
              self.send(input.into_bytes());
              Repl::display_carrot(will_output);
            }
          }
        },
        Err(err) => panic!("Error reading from stdin: {}", err),
      }
    }
  }

  fn read_line(buffer: &mut String) -> std::io::Result<usize> {
    let ret = std::io::stdin().read_line(buffer);
    ret
  }

  fn display_carrot(newline: bool) {
    if newline { print!("\n") };
    print!("bf> ");
    match std::io::stdout().flush() {
      Ok(_) => {},
      Err(err) => panic!("Error flushing stdout: {}", err),
    }
  }

  fn send(&mut self, data: Vec<u8>) {
    match self.tx.send(data) {
      Ok(_) => {},
      Err(_) => {
        self.running = false;
        return;
      }
    }
    self.barrier.wait();
  }

  fn interpret_command(&mut self, command: &str) -> bool {
    let lowercase = command.trim().to_lowercase();
    let will_output = lowercase.contains(".");
    match lowercase.as_ref() {
      "quit" => {
        self.send(vec![EOF]);
        self.running = false;
        false
      },
      _ => will_output,
    }
  }
}

pub struct Context {
  tape: Vec<u8>,
  current_index: usize,
  input_buffer: VecDeque<u8>,
}

impl Context {
  pub fn new() -> Self {
    Context {
      tape: vec![0; 1],
      current_index: 0,
      input_buffer: VecDeque::new(),
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

  pub fn input(&mut self) {
    
    loop {
      match self.input_buffer.pop_front() {
        Some(input) => {
          self.write(input);
          break;
        }
        None => self.input_buffer.append(&mut read_input()),
      }
    }
  }

  pub fn output(&self) {
    print!("{}", char::from(self.read()));
    match std::io::stdout().flush() {
      Ok(_) => {},
      Err(err) => println!("Error flushing the output buffer: {}", err),
    }
  }

  fn write(&mut self, value: u8) {
    self.tape[self.current_index] = value;
  }

  fn read(&self) -> u8 {
    self.tape[self.current_index]
  }

  pub fn increment(&mut self) {
    let old = self.tape[self.current_index];
    self.tape[self.current_index] = old.wrapping_add(1);
  }

  pub fn decrement(&mut self) {
    let old = self.tape[self.current_index];
    self.tape[self.current_index] = old.wrapping_sub(1);
  }

  pub fn current_cell_is_zero(&mut self) -> bool {
    self.tape[self.current_index] == 0
  }
}

fn read_input() -> VecDeque<u8> {
  let mut buffer = String::new();
  match std::io::stdin().read_line(&mut buffer) {
    Ok(_) => {},
    Err(err) => panic!("Error reading from stdin: {}", err),
  }
  let bytes = buffer.into_bytes();
  let mut ret = VecDeque::new();
  for byte in bytes.into_iter() {
    ret.push_back(byte);
  }
  ret
}
