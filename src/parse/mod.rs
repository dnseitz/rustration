
//! Parsing Brainfuck programs.
//! 
//! Because lexing is so simple for this language, the lexer runs lazily, only lexing the input as
//! a new token is required. This means the lexing and parsing stages are very closely linked as
//! the parser and lexer run essentially in lockstep.
//! 
//! The parser constructs an AST of the Brainfuck program. Each program can be represented by one
//! single block statement that contains all the expressions in the program stored sequentially.
//! 
//! Because each expression is so simple, code can easily be executed at parse time. This means
//! that as the program is being parsed it can be executed on a virtual machine. This is what
//! allows the REPL functionality of this interpreter.

use interpreter::Context;
use self::ast::{Expr, Loop, Block};
use self::token::{MetaToken, Token};
use std::sync::{Arc, Barrier};
use std::sync::mpsc::Receiver;
pub use self::token::EOF;

mod ast;
mod token;

/// A structure representing a Brainfuck program
// TODO: Break this into Code and Program?
// Code would represent the unparsed code while Program is the parsed code ready to be executed.
pub struct Code {
  code: Vec<u8>,
  current_index: usize,
  entry: Option<Block>,
  nesting: usize,

  line_num: usize,
  char_num: usize,
  // TODO: Better name
  rx: Option<Receiver<Vec<u8>>>,
  barrier: Option<Arc<Barrier>>,
}

impl Code {
  /// Feed in a vector of bytes to be parsed
  pub fn new(data: Vec<u8>) -> Self {
    Code {
      code: data,
      current_index: 0,
      entry: None,
      nesting: 0,
      line_num: 1,
      char_num: 1,
      rx: None,
      barrier: None,
    }
  }

  /// Used for the REPL interpreter, data is sent over the `rx` channel as it is recieved
  pub fn new_from_channel(rx: Receiver<Vec<u8>>, barrier: Arc<Barrier>) -> Self {
    Code {
      code: Vec::new(),
      current_index: 0,
      entry: None,
      nesting: 0,
      line_num: 1,
      char_num: 1,
      rx: Some(rx),
      barrier: Some(barrier),
    }
  }

  /// Get the next token in the stream of program data.
  fn next_token(&mut self) -> MetaToken {
    let mut len = self.code.len();
    loop {
      if self.current_index < len {
        let raw_token = self.code[self.current_index];
        let token = Token::from(self.code[self.current_index]);
        let ret = MetaToken::new(token, self.line_num, self.char_num);
        self.current_index += 1;
        if raw_token == b'\n' {
          self.line_num += 1;
          self.char_num = 0;
        }
        self.char_num += 1;
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
          return MetaToken::new(Token::Eof, self.line_num, self.char_num);
        }
      }
    }
  }

  /// Parse the program.
  pub fn parse(&mut self) {
    let entry = parse(self, false);
    self.entry = Some(entry);
  }

  /// Parse the program and execute the code as it is being parsed.
  pub fn parse_and_run(&mut self) {
    let entry = parse(self, true);
    if let Some(barrier) = self.barrier.as_ref() {
      barrier.wait();
    }
    println!("Code done parsing!");
    self.entry = Some(entry);
  }

  /// Run the already parsed program.
  pub fn run(&mut self) {
    let mut context = Context::new();
    if let Some(entry) = self.entry.as_mut() {
      entry.run(&mut context);
    }
  }
}

impl Drop for Code {
  fn drop(&mut self) {
    // TODO: Come back and look at this... this seems incredibly unsafe...
    // I'm doing it right now because if the parsing thread panics while running the REPL
    // interpreter it causes the program to hang because it's waiting for a barrier signal.
    if let Some(barrier) = self.barrier.as_ref() {
      barrier.wait();
    }
  }
}

/// Loop through each byte of data given for a program and parse it into our AST.
/// 
/// Optionaly execute the expressions as they are evaluated.
fn parse(code: &mut Code, run: bool) -> Block {
  let mut block = Block::new();
  let mut context = Context::new();

  let mut line = None;
  let mut character = None;
  loop {
    let meta_token = code.next_token();
    if line.is_none() {
      line = Some(meta_token.line());

      // Because we've already parsed the `JumpForward` token, the first token we read in this new
      // pass will be the very next character. Since there's no way we could be on a newline we
      // don't have to worry about the line number being off, but our character number will be one
      // too far...
      character = Some(meta_token.character() - 1);
    }
    let token = meta_token.token();
    let expr = match *token {
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
          panic!("Unmatched '[' starting at line: {} character: {}", 
            line.unwrap(), character.unwrap());
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

#[cfg(test)]
mod tests {
  use super::*;
  use super::token::Token;
  use std::time::{Instant, Duration};

  #[test]
  #[ignore]
  fn bench_parse() {
    const NUM_TESTS: u32 = 1000;
    let mut sum = Duration::new(0, 0);
    let data = ::read_file("test_files/life.b");

    for _ in 0..NUM_TESTS {
      let mut code = Code::new(data.clone());

      let start = Instant::now();
      code.parse();
      let end = Instant::now();
      let duration = end.duration_since(start);
      sum += duration;
    }
    sum /= NUM_TESTS;
    println!("Avg Seconds: {}, Avg Nanoseconds: {}", sum.as_secs(), sum.subsec_nanos());
  }

  #[test]
  fn code_parse() {
    let mut code = Code::new(vec![b'>', b'<', b'+', b'[', b'-', b']', b'+', b'.']);
    assert!(code.entry.is_none());

    code.parse();

    assert!(code.entry.is_some());
  }

  #[test]
  #[should_panic]
  fn code_invalid_parse_panics() {
    let mut code = Code::new(vec![b'[', b'+']);
    
    code.parse();
  }

  #[test]
  fn code_next_token() {
    let mut code = Code::new(vec![b'>', b'<', b'+', b'+', b'.']);

    let mut token;

    token = code.next_token();
    assert_eq!(token.token(), &Token::MoveRight);

    token = code.next_token();
    assert_eq!(token.token(), &Token::MoveLeft);

    token = code.next_token();
    assert_eq!(token.token(), &Token::Increment);

    token = code.next_token();
    assert_eq!(token.token(), &Token::Increment);

    token = code.next_token();
    assert_eq!(token.token(), &Token::Output);

    token = code.next_token();
    assert_eq!(token.token(), &Token::Eof);
    token = code.next_token();
    assert_eq!(token.token(), &Token::Eof);
    token = code.next_token();
    assert_eq!(token.token(), &Token::Eof);
  }
}
