
//! Abstract Syntax Tree of the Rustration interpreter
//! 
//! This module contains representations of the abstract syntax tree for the Brainfuck interpreter.
//! The AST of the Brainfuck language is fairly trivial as there are no complex conditionals or
//! jumps. There is only one statement in the tree, a `Block`, which contains a list of expressions
//! in the order that they appear. 
//! 
//! Expressions represent the different commands available in the
//! language. These include `MoveRight`, `MoveLeft`, `Increment`, `Decrement`, `Output`, `Input`,
//! and `Loop`. The majority of these expressions are trivial, only representing the operation that
//! they would perform, the only slightly complex expression is the `Loop` expression. This
//! expression contains a single `Block` statement that represents all the expressions that the
//! `Loop` should iterate over.
//! 
//! The `Loop` construct in Brainfuck is very similar to a while loop in any C-like language, and
//! so it can be modeled in a very similar way. Something like:
//! ```c
//! while *data != 0 
//! {
//!   // Execute block of expressions
//! }
//! ```
//! Would be a C representation of the Brainfuck loop.

use super::parsing::parse;
use super::parsing::Parser;
use interpreter::Context;
use std::collections::VecDeque;

/// Expressions
/// 
/// Each expression captures the semantics of each Brainfuck command. For example, the `MoveRight`
/// expression would move the data pointer one cell to the right on the virtual tape used by the
/// interpreter.
#[derive(Debug)]
pub enum Expr {
  /// Representing the '>' command, move the data pointer right one cell
  MoveRight,
  
  /// Representing the '<' command, move the data pointer left one cell
  MoveLeft,
  
  /// Representing the '+' command, increment the cell under the data pointer by one
  Increment,

  /// Representing the '-' command, decrement the cell under the data pointer by one
  Decrement,

  /// Representing the '.' command, output the value stored in the cell under the data pointer
  Output,

  /// Representing the ',' command, take a value as input and store it in the cell under the data
  /// pointer
  Input,
  
  /// Representing the loop contruct in Brainfuck, a pair of matching '[' and ']'.
  /// 
  /// At the start of the loop, the cell under the data pointer is evaluated, if it is equal to 0
  /// then the instruction pointer jumps past the matching ']', otherwise it executes the next
  /// command.
  /// 
  /// At the end of the loop, the cell under the data pointer is evaluated, if it is not equal to 0
  /// then the instruction pointer jumps back the the beginning '[', otherwise it executes the next
  /// command.
  Loop(Loop),
}

impl Expr {
  /// Execute the semantics of `self`'s variant in the given context.
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

/// A struct representing a parsed Brainfuck program.
pub struct Program {
  entry: Block,
}

impl Program {
  pub fn new(entry: Block) -> Self {
    Program { entry: entry }
  }

  /// Run the already parsed program.
  pub fn run(&self) {
    let mut context = Context::new();
    self.entry.run(&mut context);
  }
}

/// A statement enclosing a series of expressions in order.
/// 
/// When this is evaluated it executes each expression stored in order from the start of the block
/// to the end.
#[derive(Debug)]
pub struct Block {
  block: VecDeque<Expr>,
}

impl Block {
  /// Create a new, empty `Block` statement.
  pub fn new() -> Self {
    Block { block: VecDeque::new() }
  }

  /// Add an `Expr` to the block.
  /// 
  /// This expression is stored after any expressions already within the block.
  pub fn add_expr(&mut self, expr: Expr) {
    self.block.push_back(expr);
  }

  /// Execute all expressions stored in the block.
  pub fn run(&self, context: &mut Context) {
    for expr in self.block.iter() {
      expr.run(context);
    }
  }
}

/// A loop structure that stores a `Block` of the code that the loop should execute.
#[derive(Debug)]
pub struct Loop {
  block: Block,
}

impl Loop {
  /// Create a new `Loop`, parsing all the tokens stored after the initial '[' up until a matching
  /// ']' is found.
  pub fn new<T: Parser>(code: &mut T) -> super::Result<Self> {
    let block = try!(parse(code, false));
    Ok(Loop { block: block })
  }

  /// Execute the expressions within the loop as long as the conditions for looping are met.
  fn run(&self, context: &mut Context) {
    while !context.current_cell_is_zero() {
      self.block.run(context);
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use parse::Parser;

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
    let mut code = Parser::new(vec![b'>', b']']);

    let loop_expr = Loop::new(&mut code);
    //assert_eq!(loop_expr.block.block.len(), 1);
  }

  #[test]
  #[should_panic]
  fn non_matching_loop_panics() {
    let mut code = Parser::new(vec![b'>', b'<']);

    let _loop_expr = Loop::new(&mut code);
  }
}
