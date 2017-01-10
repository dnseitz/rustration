
//! Brainfuck tokens.
//! 
//! This module contains representations of the different tokens in the Brainfuck language. This is
//! a very small set of tokens, making for a very simple lexer. As there are no keywords, only one
//! character long tokens, lexing can be done in one pass without any lookahead.

/// Value marking the end of the Brainfuck file.
pub const EOF: u8 = 255;

/// Metadata wrapping type.
/// 
/// This type wraps a `Token` type with metadata about the positioning of the token in the file,
/// like line number and column number.
#[derive(Debug)]
pub struct MetaToken {
  token: Token,
  line: usize,
  character: usize,
}

impl MetaToken {
  /// Create a new `MetaToken` wrapping the corresponding `Token` at the specified line and column.
  pub fn new(token: Token, line: usize, character: usize) -> Self {
    MetaToken {
      token: token,
      line: line,
      character: character,
    }
  }

  /// Get a reference to the token.
  pub fn token(&self) -> &Token {
    &self.token
  }

  /// Get the line number of the token.
  pub fn line(&self) -> usize {
    self.line
  }

  /// Get the column number of the token.
  pub fn character(&self) -> usize {
    self.character
  }
}

/// Token types
/// 
/// These tokens represent the different operations available in Brainfuck. The different commands
/// are '>', '<', '+', '-', '.', ',', '[', and ']'. All other characters are considered comments.
#[derive(Debug, PartialEq)]
pub enum Token {
  /// The '>' character, move the data pointer right one cell.
  MoveRight,

  /// The '<' character, move the data pointer left one cell.
  MoveLeft,

  /// The '+' character, increment the value under the data pointer.
  Increment,

  /// The '-' character, decrement the value under the data pointer.
  Decrement,

  /// The '.' character, output the value stored under the data pointer.
  Output,
  
  /// The ',' character, read in a data value and store it at the cell under the data pointer.
  Input,

  /// The '[' character, jump past the matching ']' if the cell under the data pointer is 0.
  JumpForward,
  
  /// The ']' character, jump back to the matching '[' if the cell under the data pointer is not 0.
  JumpBack,
  
  /// Any other character, these are ignored and have no effect.
  Comment,

  /// A variant marking the end of a Brainfuck file.
  Eof,
}

impl From<u8> for Token {
  fn from(data: u8) -> Self {
    match data {
      b'>' => Token::MoveRight,
      b'<' => Token::MoveLeft,
      b'+' => Token::Increment,
      b'-' => Token::Decrement,
      b'.' => Token::Output,
      b',' => Token::Input,
      b'[' => Token::JumpForward,
      b']' => Token::JumpBack,
      EOF  => Token::Eof,
      _    => Token::Comment,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn move_right_token() {
    let tok = Token::from(b'>');
    let tok2: Token = b'>'.into();

    assert_eq!(tok, Token::MoveRight);
    assert_eq!(tok2, Token::MoveRight);
  }

  #[test]
  fn move_left_token() {
    let tok: Token = Token::from(b'<');
    let tok2: Token = b'<'.into();

    assert_eq!(tok, Token::MoveLeft);
    assert_eq!(tok2, Token::MoveLeft);
  }

  #[test]
  fn increment_token() {
    let tok: Token = Token::from(b'+');
    let tok2: Token = b'+'.into();

    assert_eq!(tok, Token::Increment);
    assert_eq!(tok2, Token::Increment);
  }

  #[test]
  fn decrement_token() {
    let tok: Token = Token::from(b'-');
    let tok2: Token = b'-'.into();

    assert_eq!(tok, Token::Decrement);
    assert_eq!(tok2, Token::Decrement);
  }

  #[test]
  fn output_token() {
    let tok: Token = Token::from(b'.');
    let tok2: Token = b'.'.into();

    assert_eq!(tok, Token::Output);
    assert_eq!(tok2, Token::Output);
  }

  #[test]
  fn input_token() {
    let tok: Token = Token::from(b',');
    let tok2: Token = b','.into();

    assert_eq!(tok, Token::Input);
    assert_eq!(tok2, Token::Input);
  }

  #[test]
  fn jump_forward_token() {
    let tok: Token = Token::from(b'[');
    let tok2: Token = b'['.into();

    assert_eq!(tok, Token::JumpForward);
    assert_eq!(tok2, Token::JumpForward);
  }

  #[test]
  fn jump_back_token() {
    let tok: Token = Token::from(b']');
    let tok2: Token = b']'.into();

    assert_eq!(tok, Token::JumpBack);
    assert_eq!(tok2, Token::JumpBack);
  }

  #[test]
  fn eof_token() {
    let tok: Token = Token::from(EOF);
    let tok2: Token = EOF.into();

    assert_eq!(tok, Token::Eof);
    assert_eq!(tok2, Token::Eof);
  }

  #[test]
  fn comment_token() {
    let tok: Token = Token::from(b'a');
    let tok2: Token = b'b'.into();

    assert_eq!(tok, Token::Comment);
    assert_eq!(tok2, Token::Comment);
  }
}
