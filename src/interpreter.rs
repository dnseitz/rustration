
//! Brainfuck REPL Interpreter
//! 
//! The REPL interpreter allows you to input a Brainfuck program from the command line and see it
//! execute on the fly.
//! 
//! This module also contains the context of the virtual machine used to execute Brainfuck code.

use std;
use std::str::FromStr;
use std::io::{Write};
use std::sync::mpsc::{Sender, Receiver};
use std::collections::VecDeque;
use parse::EOF;
use parse::ReplParser;

enum Command {
  Quit,
  Interpret(String),
}

impl FromStr for Command {
  type Err = ();

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let old = s;
    match s.trim() {
      "quit" => Ok(Command::Quit),
      _ => {
        Ok(Command::Interpret(old.into()))
      },
    }
  }
}

pub enum Status {
  /// The parsing thread is ready for more input
  Ready,

  /// The parsing thread has exited, this is likely because of a parsing error
  Exited,
}

/// A REPL interpreter that takes input from the command line and executes it.
/// 
/// Input can be any valid ascii characters, the Brainfuck interpreter will ignore any non command
/// characters and execute any command characters it recieves. There are some keywords that are
/// used as commands to the REPL interpreter like `quit` which stops the interpreter.
pub struct Repl {
  data_channel: Sender<Vec<u8>>,
  status_channel: Receiver<Status>,
  running: bool,
}

impl Repl {
  /// Create a new REPL interpreter ready to be run.
  pub fn new() -> Self {
    let (data_tx, data_rx) = std::sync::mpsc::channel();
    let (status_tx, status_rx) = std::sync::mpsc::channel();
    let mut code = ReplParser::new(data_rx, status_tx);
    let _handle = std::thread::Builder::new()
      .name(String::from("parse"))
      .spawn(move|| {
        match code.parse_and_run() {
          Ok(_) => {},
          Err(err) => println!("{}", err),
        }
      });

    Repl {
      data_channel: data_tx,
      status_channel: status_rx,
      running: false,
    }
  }

  /// Start running the REPL interpreter.
  pub fn start(&mut self) {
    self.running = true;
    Repl::display_carrot(false);
    self.wait_for_status();
    while self.running {
      let input = self.read_line();
      match input {
        Some(input) => {
            // parse -> Command cannot fail
            let command = input.parse().unwrap();
            self.interpret_command(command);
        },
        None => self.exit(),
      }
    }
  }
  
  fn wait_for_status(&mut self) {
    match self.status_channel.recv() {
      Ok(status) => match status {
        Status::Ready => {},
        Status::Exited => self.exit(),
      },
      Err(_) => {
        self.exit();
      },
    }
  }

  fn read_line(&mut self) -> Option<String> {
    let mut buffer = String::new();
    let num_read = match std::io::stdin().read_line(&mut buffer) {
      Ok(num_read) => num_read,
      Err(err) => panic!("Error reading from stdin: {}", err),
    };
    if num_read == 0 { None } else { Some(buffer) }
  }

  fn exit(&mut self) {
    self.send(vec![EOF]);
    self.running = false;
  }

  fn display_carrot(newline: bool) {
    if newline { print!("\n") };
    print!("bf> ");
    if let Err(err) = std::io::stdout().flush() {
      panic!("Error flushing stdout: {}", err);
    }
  }

  fn send(&mut self, data: Vec<u8>) {
    if let Err(_) = self.data_channel.send(data) {
      self.running = false;
      return;
    }
    // Wait for the parse thread to parse and execute
    self.wait_for_status();
  }

  fn interpret_command(&mut self, command: Command) {
    match command {
      Command::Quit => self.exit(),
      Command::Interpret(input) => {
        let will_output = input.contains(".");
        self.send(input.into_bytes());
        Repl::display_carrot(will_output);
      },
    }
  }
}

/// The context of a virtual machine to run a Brainfuck program on.
pub struct Context {
  tape: Vec<i8>,
  current_index: usize,
  input_buffer: VecDeque<u8>,
}

impl Context {
  /// Create a new, fresh context with an empty tape and empty input buffer.
  pub fn new() -> Self {
    Context {
      tape: vec![0; 1],
      current_index: 0,
      input_buffer: VecDeque::new(),
    }
  }

  /// Move the data pointer right one.
  pub fn move_right(&mut self) {
    self.current_index += 1;
    while self.current_index >= self.tape.len() {
      self.tape.push(0);
    }
  }

  /// Move the data pointer left one.
  pub fn move_left(&mut self) {
    if self.current_index > 0 {
      self.current_index -= 1;
    }
  }

  /// Retrieve input from the input buffer or the command line if the input buffer is empty.
  pub fn input(&mut self) {
    loop {
      match self.input_buffer.pop_front() {
        Some(input) => {
          self.write(input as i8);
          break;
        }
        None => self.input_buffer.append(&mut read_input()),
      }
    }
  }

  /// Output the value stored under the data pointer.
  pub fn output(&self) {
    print!("{}", char::from(self.read() as u8));
    match std::io::stdout().flush() {
      Ok(_) => {},
      Err(err) => println!("Error flushing the output buffer: {}", err),
    }
  }

  fn write(&mut self, value: i8) {
    self.tape[self.current_index] = value;
  }

  fn read(&self) -> i8 {
    self.tape[self.current_index]
  }

  /// Increment the value stored in the cell under the data pointer.
  pub fn increment(&mut self) {
    let old = self.tape[self.current_index];
    self.tape[self.current_index] = old.wrapping_add(1);
  }

  /// Decrement the value stored in the cell under the data pointer.
  pub fn decrement(&mut self) {
    let old = self.tape[self.current_index];
    self.tape[self.current_index] = old.wrapping_sub(1);
  }

  /// Return true if the value stored in the cell under the data pointer is zero, false otherwise.
  pub fn current_cell_is_zero(&mut self) -> bool {
    self.tape[self.current_index] == 0
  }
}

/// Read input from the command line
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
