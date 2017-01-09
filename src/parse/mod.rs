
use interpreter::Context;
use self::ast::{Expr, Loop, Block};
use self::token::Token;

mod ast;
mod token;

#[derive(Debug)]
pub struct Code {
  code: Vec<u8>,
  current_index: usize,
  entry: Option<Block>,
}

impl Code {
  pub fn new(data: Vec<u8>) -> Self {
    Code {
      code: data,
      current_index: 0,
      entry: None,
    }
  }

  pub fn next_token(&mut self) -> Option<Token> {
    if self.current_index < self.code.len() {
      let ret = Some(Token::from(self.code[self.current_index]));
      self.current_index += 1;
      ret
    }
    else {
      None
    }
  }

  pub fn parse(&mut self) {
    let entry = parse(self);
    self.entry = Some(entry);
  }

  pub fn run(&mut self) {
    let mut context = Context::new();
    if let Some(entry) = self.entry.as_mut() {
      entry.run(&mut context);
    }
  }
}

fn parse(code: &mut Code) -> Block {
  let mut block = Block::new();
  while let Some(token) = code.next_token() {
    match token {
      Token::MoveRight => block.add_expr(Expr::MoveRight),
      Token::MoveLeft => block.add_expr(Expr::MoveLeft),
      Token::Increment => block.add_expr(Expr::Increment),
      Token::Decrement => block.add_expr(Expr::Decrement),
      Token::Output => block.add_expr(Expr::Output),
      Token::Input => block.add_expr(Expr::Input),
      Token::JumpForward => block.add_expr(Expr::Loop(Loop::new(code))),
      Token::JumpBack => {
        return block;
      },
      Token::Comment => { /* no-op */ },
    }
  }
  block
}
