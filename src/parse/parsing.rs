// parse/parsing.rs
// Rustration
//
// Created by Daniel Seitz on 11/11/17

use super::error::ParseError;
use super::Result;
use super::ast::{Program, Expr, Loop, Block};
use super::token::{MetaToken, Token};
use interpreter::{Context, Status};
use std::sync::mpsc::{Sender, Receiver};

/*
trait Code {

}

pub struct ReplCode {
  code: RawCode,
  data_channel: Option<Receiver<Vec<u8>>>,
  status_channel: Option<Sender<Status>>,
}
*/

/// A structure representing a Brainfuck program
pub struct Code {
  code: Vec<u8>,
  current_index: usize,
  nesting: usize,

  line_num: usize,
  char_num: usize,
  // TODO: Better name
  data_channel: Option<Receiver<Vec<u8>>>,
  status_channel: Option<Sender<Status>>,
  //barrier: Option<Arc<Barrier>>,
}

impl Code {
  /// Feed in a vector of bytes to be parsed
  pub fn new(data: Vec<u8>) -> Self {
    Code {
      code: data,
      current_index: 0,
      nesting: 0,
      line_num: 1,
      char_num: 1,
      data_channel: None,
      status_channel: None,
      //barrier: None,
    }
  }

  /// Used for the REPL interpreter, data is sent over the `rx` channel as it is recieved
  pub fn new_from_channel(data_channel: Receiver<Vec<u8>>, status_channel: Sender<Status>) -> Self {
    Code {
      code: Vec::new(),
      current_index: 0,
      nesting: 0,
      line_num: 1,
      char_num: 1,
      data_channel: Some(data_channel),
      status_channel: Some(status_channel),
      //barrier: Some(barrier),
    }
  }

  fn eof_token(&self) -> MetaToken {
    MetaToken::new(Token::Eof, self.line_num, self.char_num)
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
        // Wait for more data
        match (self.data_channel.as_ref(), self.status_channel.as_ref()) {
          (Some(data_channel), Some(status_channel)) => {
            if let Err(_) = status_channel.send(Status::Ready) {
              return self.eof_token();
            }
            if let Some(mut new_code) = data_channel.recv().ok() {
              self.code.append(&mut new_code);
              len = self.code.len();
            }
            else {
              return self.eof_token();
            }
          },
          (None, None) => {
            return self.eof_token();
          },
          (_, _) => unreachable!(),
        }
      }
    }
  }

  /// Parse the program.
  pub fn parse(&mut self) -> Result<Program> {
    let entry = try!(parse(self, false));
    Ok(Program::new(entry))
  }

  /// Parse the program and execute the code as it is being parsed.
  pub fn parse_and_run(&mut self) -> Result<Program> {
    let entry = parse(self, true);
    if let Some(ref status_channel) = self.status_channel {
      status_channel.send(Status::Exited).ok();
    }
    entry.map(Program::new)
  }

}

/*
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
*/

/// Loop through each byte of data given for a program and parse it into our AST.
/// 
/// Optionaly execute the expressions as they are evaluated.
pub fn parse(code: &mut Code, run: bool) -> Result<Block> {
  let mut block = Block::new();
  let mut context = Context::new();

  let mut start_line = None;
  let mut start_char = None;
  loop {
    let meta_token = code.next_token();
    let line = meta_token.line();
    let character = meta_token.character();
    if start_line.is_none() {
      start_line = Some(line);

      // Because we've already parsed the `JumpForward` token, the first token we read in this new
      // pass will be the very next character. Since there's no way we could be on a newline we
      // don't have to worry about the line number being off, but our character number will be one
      // too far...
      start_char = Some(character - 1);
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
        Expr::Loop(try!(Loop::new(code)))
      },
      Token::JumpBack => {
        if code.nesting == 0 {
          return Err(ParseError::UnmatchedCloseBrace(line, character));
        }
        code.nesting -= 1;
        return Ok(block);
      },
      Token::Comment => continue,
      Token::Eof => {
        if code.nesting > 0 {
          return Err(ParseError::UnmatchedOpenBrace(start_line.unwrap(), start_char.unwrap()));
        }
        break;
      }
    };

    if code.nesting == 0 && run {
      expr.run(&mut context);
    }
    block.add_expr(expr);
  }
  Ok(block)
}

#[cfg(test)]
mod tests {
  use super::*;
  use parse::token::Token;
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

    code.parse();
  }

  #[test]
  fn code_invalid_parse_panics() {
    let mut code = Code::new(vec![b'[', b'+']);
    
    assert!(code.parse().is_err());
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
