
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
