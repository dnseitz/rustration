
use std;
use std::io::Read;
use super::parse;
use super::Code;
use interpreter::Context;
use std::collections::VecDeque;

#[derive(Debug)]
pub enum Expr {
  MoveRight,
  MoveLeft,
  Increment,
  Decrement,
  Output,
  Input,
  Loop(Loop),
}

impl Expr {
  fn run(&mut self, context: &mut Context) {
    match *self {
      Expr::MoveRight => context.move_right(),
      Expr::MoveLeft => context.move_left(),
      Expr::Increment => context.increment(),
      Expr::Decrement => context.decrement(),
      Expr::Output => print!("{}", char::from(context.read() as u8)),
      Expr::Input => context.write(read_input()),
      Expr::Loop(ref mut inner) => inner.run(context),
    }
  }
}

#[derive(Debug)]
pub struct Block {
  block: VecDeque<Expr>,
}

impl Block {
  pub fn new() -> Self {
    Block { block: VecDeque::new() }
  }

  pub fn add_expr(&mut self, expr: Expr) {
    self.block.push_back(expr);
  }

  pub fn run(&mut self, context: &mut Context) {
    for expr in self.block.iter_mut() {
      expr.run(context);
    }
  }
}

#[derive(Debug)]
pub struct Loop {
  block: Block,
}

impl Loop {
  pub fn new(code: &mut Code) -> Self {
    Loop { block: parse(code) }
  }

  fn run(&mut self, context: &mut Context) {
    while !context.current_cell_is_zero() {
      self.block.run(context);
    }
  }
}

fn read_input() -> isize {
  std::io::stdin()
    .bytes()
    .next()
    .and_then(|result| result.ok())
    .map(|byte| byte as isize)
    .unwrap()
}
