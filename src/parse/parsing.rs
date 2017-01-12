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

pub trait Parser {
  fn next_token(&mut self) -> MetaToken;

  fn increment_nest_level(&mut self);
  fn decrement_nest_level(&mut self);
  fn nest_level(&self) -> usize;
}

pub struct ReplParser {
  code: RawParser,
  data_channel: Receiver<Vec<u8>>,
  status_channel: Sender<Status>,
}

impl Parser for ReplParser {
  /// Get the next token in the stream of program data.
  fn next_token(&mut self) -> MetaToken {
    loop {
      let next_token = self.code.next_token();
      match *next_token.token() {
        Token::Eof => {
          if let Err(_) = self.status_channel.send(Status::Ready) {
            return self.code.eof_token();
          }
          match self.data_channel.recv().ok() {
            Some(mut new_code) => self.code.code.append(&mut new_code),
            None => return self.code.eof_token(),
          }
        },
        _ => return next_token,
      }
    }
  }
  
  fn increment_nest_level(&mut self) {
    self.code.increment_nest_level();
  }

  fn decrement_nest_level(&mut self) {
    self.code.decrement_nest_level();
  }

  fn nest_level(&self) -> usize {
    self.code.nest_level()
  }
}

impl ReplParser {
  /// Used for the REPL interpreter, data is sent over the `rx` channel as it is recieved
  pub fn new(data_channel: Receiver<Vec<u8>>, status_channel: Sender<Status>) -> Self {
    ReplParser {
      code: RawParser::new(Vec::new()),
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
  fn next_token(&mut self) -> MetaToken {
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
      ret
    }
    else {
      self.eof_token()
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
pub fn parse<T: Parser>(code: &mut T, run: bool) -> Result<Block> {
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
        code.increment_nest_level();
        Expr::Loop(try!(Loop::new(code)))
      },
      Token::JumpBack => {
        if code.nest_level() == 0 {
          return Err(ParseError::UnmatchedCloseBrace(line, character));
        }
        code.decrement_nest_level();
        return Ok(block);
      },
      Token::Comment => continue,
      Token::Eof => {
        if code.nest_level() > 0 {
          return Err(ParseError::UnmatchedOpenBrace(start_line.unwrap(), start_char.unwrap()));
        }
        break;
      }
    };

    if code.nest_level() == 0 && run {
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
      let mut code = Parser::new(data.clone());

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
    let mut code = Parser::new(vec![b'>', b'<', b'+', b'[', b'-', b']', b'+', b'.']);

    code.parse();
  }

  #[test]
  fn code_invalid_parse_panics() {
    let mut code = Parser::new(vec![b'[', b'+']);
    
    assert!(code.parse().is_err());
  }

  #[test]
  fn code_next_token() {
    let mut code = Parser::new(vec![b'>', b'<', b'+', b'+', b'.']);

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
