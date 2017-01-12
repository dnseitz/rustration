// parse/error.rs
// Rustration
//
// Created by Daniel Seitz on 1/11/17

use std;
use std::error::Error;

#[derive(Debug)]
pub enum ParseError {
  UnmatchedOpenBrace(usize, usize),
  UnmatchedCloseBrace(usize, usize),
}

impl std::fmt::Display for ParseError {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    match *self {
      ParseError::UnmatchedOpenBrace(line, column) => {
        write!(f, "Unmatched '[' starting at line: {}, column: {}", line, column)
      },
      ParseError::UnmatchedCloseBrace(line, column) => {
        write!(f, "Unmatched ']' starting at line: {}, column: {}", line, column)
      },
    }
  }
}

impl Error for ParseError {
  fn description(&self) -> &str {
    match *self {
      ParseError::UnmatchedOpenBrace(..) => "Unmatched '['",
      ParseError::UnmatchedCloseBrace(..) => "Unmatched ']'",
    }
  }

  fn cause(&self) -> Option<&Error> {
    None
  }
}
