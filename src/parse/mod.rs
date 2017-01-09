
use interpreter::Context;
use self::ast::{Expr, Loop, Block};
use self::token::Token;
use std::sync::{Arc, Barrier};
use std::sync::mpsc::Receiver;
pub use self::token::EOF;

mod ast;
mod token;

pub struct Code {
  code: Vec<u8>,
  current_index: usize,
  entry: Option<Block>,
  nesting: usize,
  // TODO: Better name
  rx: Option<Receiver<Vec<u8>>>,
  barrier: Option<Arc<Barrier>>,
}

impl Code {
  pub fn new(data: Vec<u8>) -> Self {
    Code {
      code: data,
      current_index: 0,
      entry: None,
      nesting: 0,
      rx: None,
      barrier: None,
    }
  }

  pub fn new_from_channel(rx: Receiver<Vec<u8>>, barrier: Arc<Barrier>) -> Self {
    Code {
      code: Vec::new(),
      current_index: 0,
      entry: None,
      nesting: 0,
      rx: Some(rx),
      barrier: Some(barrier),
    }
  }

  fn next_token(&mut self) -> Token {
    let mut len = self.code.len();
    loop {
      if self.current_index < len {
        let ret = Token::from(self.code[self.current_index]);
        self.current_index += 1;
        return ret
      }
      else {
        if let Some(rx) = self.rx.as_ref() {
          self.barrier.as_ref().expect("Barrier should not be None").wait();
          let mut new_code = rx.recv().unwrap();

          self.code.append(&mut new_code);
          len = self.code.len();
        }
        else {
          return Token::Eof;
        }
      }
    }
  }

  pub fn parse(&mut self) {
    let entry = parse(self, false);
    self.entry = Some(entry);
  }

  pub fn parse_and_run(&mut self) {
    let entry = parse(self, true);
    if let Some(barrier) = self.barrier.as_ref() {
      barrier.wait();
    }
    println!("Code done parsing!");
    self.entry = Some(entry);
  }

  pub fn run(&mut self) {
    let mut context = Context::new();
    if let Some(entry) = self.entry.as_mut() {
      entry.run(&mut context);
    }
  }
}

fn parse(code: &mut Code, run: bool) -> Block {
  let mut block = Block::new();
  let mut context = Context::new();
  loop {
    let token = code.next_token();
    let expr = match token {
      Token::MoveRight => Expr::MoveRight,
      Token::MoveLeft => Expr::MoveLeft,
      Token::Increment => Expr::Increment,
      Token::Decrement => Expr::Decrement,
      Token::Output => Expr::Output,
      Token::Input => Expr::Input,
      Token::JumpForward => {
        code.nesting += 1;
        Expr::Loop(Loop::new(code))
      },
      Token::JumpBack => {
        code.nesting -= 1;
        return block;
      },
      Token::Comment => continue,
      Token::Eof => {
        if code.nesting > 0 {
          panic!("Unmatched '['!");
        }
        break;
      }
    };

    if code.nesting == 0 && run {
      expr.run(&mut context);
    }
    block.add_expr(expr);
  }
  block
}
