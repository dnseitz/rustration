// parse/parsing.rs
// Rustration
//
// Created by Daniel Seitz on 1/11/17

use super::error::ParseError;
use super::Result;
use super::ast::{Program, Expr, Loop, Block};
use super::token::{MetaToken, Token};
use interpreter::{Context, Status};
use std::sync::mpsc::{Sender, Receiver};

pub trait Parser {
  fn next_token(&mut self) -> Option<MetaToken>;

  fn increment_nest_level(&mut self);
  fn decrement_nest_level(&mut self);
  fn nest_level(&self) -> usize;
}

pub struct ReplParser {
  inner: RawParser,
  data_channel: Receiver<Vec<u8>>,
  status_channel: Sender<Status>,
}

impl Parser for ReplParser {
  /// Get the next token in the stream of program data.
  fn next_token(&mut self) -> Option<MetaToken> {
    loop {
      match self.inner.next_token() {
        None => {
          if let Err(_) = self.status_channel.send(Status::Ready) {
            return Some(self.inner.eof_token());
          }
          match self.data_channel.recv().ok() {
            Some(mut new_code) => self.inner.code.append(&mut new_code),
            None => return Some(self.inner.eof_token()),
          }
        },
        next_token => return next_token,
      }
    }
  }
  
  fn increment_nest_level(&mut self) {
    self.inner.increment_nest_level();
  }

  fn decrement_nest_level(&mut self) {
    self.inner.decrement_nest_level();
  }

  fn nest_level(&self) -> usize {
    self.inner.nest_level()
  }
}

impl ReplParser {
  /// Used for the REPL interpreter, data is sent over the `rx` channel as it is recieved
  pub fn new(data_channel: Receiver<Vec<u8>>, status_channel: Sender<Status>) -> Self {
    ReplParser {
      inner: RawParser::new(Vec::new()),
      data_channel: data_channel,
      status_channel: status_channel,
    }
  }

  /// Parse the program and execute the code as it is being parsed.
  pub fn parse_and_run(&mut self) -> Result<Program> {
    let entry = parse(self, true);
    self.status_channel.send(Status::Exited).ok();
    entry.map(Program::new)
  }
}

/// A structure representing a Brainfuck program
pub struct RawParser {
  code: Vec<u8>,
  current_index: usize,
  nesting: usize,

  line_num: usize,
  char_num: usize,
}

impl Parser for RawParser {
  /// Get the next token in the stream of program data.
  fn next_token(&mut self) -> Option<MetaToken> {
    if self.current_index < self.code.len() {
      let raw_token = self.code[self.current_index];
      let token = Token::from(self.code[self.current_index]);
      let ret = MetaToken::new(token, self.line_num, self.char_num);
      self.current_index += 1;
      if raw_token == b'\n' {
        self.line_num += 1;
        self.char_num = 0;
      }
      self.char_num += 1;
      Some(ret)
    }
    else {
      None 
    }
  }
  
  fn increment_nest_level(&mut self) {
    self.nesting += 1;
  }

  fn decrement_nest_level(&mut self) {
    self.nesting -= 1;
  }

  fn nest_level(&self) -> usize {
    self.nesting
  }
}

impl RawParser {
  /// Feed in a vector of bytes to be parsed
  pub fn new(data: Vec<u8>) -> Self {
    RawParser {
      code: data,
      current_index: 0,
      nesting: 0,
      line_num: 1,
      char_num: 1,
    }
  }

  fn eof_token(&self) -> MetaToken {
    MetaToken::new(Token::Eof, self.line_num, self.char_num)
  }

  /// Parse the program.
  pub fn parse(&mut self) -> Result<Program> {
    let entry = try!(parse(self, false));
    Ok(Program::new(entry))
  }

}

/// Loop through each byte of data given for a program and parse it into our AST.
/// 
/// Optionaly execute the expressions as they are evaluated.
pub fn parse<T: Parser>(parser: &mut T, run: bool) -> Result<Block> {
  let mut block = Block::new();
  let mut context = Context::new();

  let mut start_line = None;
  let mut start_char = None;
  loop {
    let meta_token = if let Some(meta_token) = parser.next_token() {
      meta_token
    }
    else {
      // 0 for line and column because we don't care about EOF
      MetaToken::new(Token::Eof, 0, 0)
    };
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
        parser.increment_nest_level();
        Expr::Loop(try!(Loop::new(parser)))
      },
      Token::JumpBack => {
        if parser.nest_level() == 0 {
          return Err(ParseError::UnmatchedCloseBrace(line, character));
        }
        parser.decrement_nest_level();
        return Ok(block);
      },
      Token::Comment => continue,
      Token::Eof => {
        if parser.nest_level() > 0 {
          return Err(ParseError::UnmatchedOpenBrace(start_line.unwrap(), start_char.unwrap()));
        }
        break;
      }
    };

    if parser.nest_level() == 0 && run {
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
  use std;

  #[test]
  #[ignore]
  fn bench_parse() {
    use std::time::{Instant, Duration};
    const NUM_TESTS: u32 = 1000;
    let mut sum = Duration::new(0, 0);
    let data = ::read_file("test_files/life.b");

    for _ in 0..NUM_TESTS {
      let mut parser = RawParser::new(data.clone());

      let start = Instant::now();
      let _program = parser.parse();
      let end = Instant::now();
      let duration = end.duration_since(start);
      sum += duration;
    }
    sum /= NUM_TESTS;
    println!("Avg Seconds: {}, Avg Nanoseconds: {}", sum.as_secs(), sum.subsec_nanos());
  }

  #[test]
  fn repl_parse() {
    let (data_tx, data_rx) = std::sync::mpsc::channel();
    let (status_tx, status_rx) = std::sync::mpsc::channel();

    let mut parser = ReplParser::new(data_rx, status_tx);
    std::thread::spawn(move|| {
      assert!(parser.parse_and_run().is_ok());
    });
    assert_eq!(status_rx.recv().unwrap(), Status::Ready);
    assert!(data_tx.send(vec![b'+', b'+', b'>', b'<']).is_ok());
    assert_eq!(status_rx.recv().unwrap(), Status::Ready);
    drop(data_tx);
    assert_eq!(status_rx.recv().unwrap(), Status::Exited);
  }

  #[test]
  fn repl_invalid_parse_errors() {
    let (data_tx, data_rx) = std::sync::mpsc::channel();
    let (status_tx, status_rx) = std::sync::mpsc::channel();

    let mut parser = ReplParser::new(data_rx, status_tx);
    std::thread::spawn(move|| {
      assert!(parser.parse_and_run().is_err());
    });
    assert_eq!(status_rx.recv().unwrap(), Status::Ready);
    assert!(data_tx.send(vec![b'+', b'+', b'>', b'<', b'[']).is_ok());
    assert_eq!(status_rx.recv().unwrap(), Status::Ready);
    drop(data_tx);
    assert_eq!(status_rx.recv().unwrap(), Status::Exited);
  }

  #[test]
  fn code_parse() {
    let mut parser = RawParser::new(vec![b'>', b'<', b'+', b'[', b'-', b']', b'+', b'.']);

    assert!(parser.parse().is_ok());
  }

  #[test]
  fn code_invalid_parse_errors() {
    let mut parser = RawParser::new(vec![b'[', b'+']);
    
    assert!(parser.parse().is_err());
  }

  #[test]
  fn code_next_token() {
    let mut parser = RawParser::new(vec![b'>', b'<', b'+', b'+', b'.']);

    let mut token;

    token = parser.next_token();
    assert_eq!(token.token(), &Token::MoveRight);

    token = parser.next_token();
    assert_eq!(token.token(), &Token::MoveLeft);

    token = parser.next_token();
    assert_eq!(token.token(), &Token::Increment);

    token = parser.next_token();
    assert_eq!(token.token(), &Token::Increment);

    token = parser.next_token();
    assert_eq!(token.token(), &Token::Output);

    token = parser.next_token();
    assert_eq!(token.token(), &Token::Eof);
    token = parser.next_token();
    assert_eq!(token.token(), &Token::Eof);
    token = parser.next_token();
    assert_eq!(token.token(), &Token::Eof);
  }
}
