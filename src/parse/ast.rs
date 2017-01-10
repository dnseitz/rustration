
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
  pub fn run(&self, context: &mut Context) {
    match *self {
      Expr::MoveRight => context.move_right(),
      Expr::MoveLeft => context.move_left(),
      Expr::Increment => context.increment(),
      Expr::Decrement => context.decrement(),
      Expr::Output => context.output(),
      Expr::Input => context.input(),
      Expr::Loop(ref inner) => inner.run(context),
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

  pub fn run(&self, context: &mut Context) {
    for expr in self.block.iter() {
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
    Loop { block: parse(code, false) }
  }

  fn run(&self, context: &mut Context) {
    while !context.current_cell_is_zero() {
      self.block.run(context);
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use parse::Code;

  #[test]
  fn add_expr_to_block() {
    let mut block = Block::new();
    assert_eq!(block.block.len(), 0);

    block.add_expr(Expr::MoveLeft);
    block.add_expr(Expr::MoveRight);

    assert_eq!(block.block.len(), 2);
  }

  #[test]
  fn generate_loop() {
    let mut code = Code::new(vec![b'>', b']']);
    code.nesting = 1;

    let loop_expr = Loop::new(&mut code);
    assert_eq!(loop_expr.block.block.len(), 1);
  }

  #[test]
  #[should_panic]
  fn non_matching_loop_panics() {
    let mut code = Code::new(vec![b'>', b'<']);
    code.nesting = 1;

    let _loop_expr = Loop::new(&mut code);
  }
}
