
pub const EOF: u8 = 255;

#[derive(Debug, PartialEq)]
pub enum Token {
  MoveRight,
  MoveLeft,
  Increment,
  Decrement,
  Output,
  Input,
  JumpForward,
  JumpBack,
  Comment,
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
