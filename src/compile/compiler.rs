// compile/mod.rs
// Rustration
//
// Created by Daniel Seitz on 1/12/17

use parse::ast::Program;
use parse::ast::Expr;
use super::bytecode::{ByteProgram, ByteCode};
use std::collections::VecDeque;
use std::fmt;

#[derive(Debug, Clone)]
pub struct Label(String);

impl fmt::Display for Label {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

pub trait Compiler {
  fn compile_program(&mut self, program: &Program) -> ByteProgram;
  fn compile_expr(&mut self, expr: &Expr) -> VecDeque<ByteCode>;
}

pub struct SimpleCompiler {
  loop_count: usize,
}

impl Compiler for SimpleCompiler {
  fn compile_program(&mut self, program: &Program) -> ByteProgram {
    let mut byte_code = program.compile(self);
    byte_code.push_back(ByteCode::Exit);
    ByteProgram::from(byte_code)

    // TODO: Drain stdin
  }

  fn compile_expr(&mut self, expr: &Expr) -> VecDeque<ByteCode> {
    let mut byte_code = VecDeque::new();
    match *expr {
      Expr::MoveRight => {  //println!("  inc rsp"),
        byte_code.push_back(ByteCode::MoveRight(1));
        byte_code
      },
      Expr::MoveLeft => { //println!("  dec rsp"),
        byte_code.push_back(ByteCode::MoveLeft(1));
        byte_code
      },
      Expr::Increment => {  //println!("  inc byte [rsp]"),
        byte_code.push_back(ByteCode::Add(1));
        byte_code
      },
      Expr::Decrement => { //println!("  dec byte [rsp]"),
        byte_code.push_back(ByteCode::Sub(1));
        byte_code
      }
      Expr::Output => { //println!(concat!("  mov rax, 0x2000004 ; write\n",
        byte_code.push_back(ByteCode::Write);
        byte_code                         //"  mov rdi, 1         ; stdout\n",
      },                                 //"  mov rsi, rsp\n",
                                       //"  mov rdx, 1\n",
                                       //"  syscall")),
      Expr::Input => {  //println!(concat!("  mov rax, 0x2000003 ; read\n",
        byte_code.push_back(ByteCode::Read);
        byte_code                       //"  mov rdi, 0         ; stdin\n",
      },                                //"  mov rsi, rsp\n",
                                      //"  mov rdx, 1\n",
                                      //"  syscall")),
      Expr::Loop(ref inner) => {
        let loop_label = self.next_loop_label();
        // TODO: Come back and fix this, I think we need to pass in a compiler kind of like the
        // context we do for interpreting
        byte_code.push_back(ByteCode::Jump(loop_label.clone()));
        //println!(concat!("  jmp _{}\n",
                         //"{}:"), 
                         //loop_label, loop_label);
        byte_code.append(&mut inner.compile(self));
        byte_code.push_back(ByteCode::JumpNotZero(loop_label));
        //println!(concat!("_{}:\n",
                         //"  cmp byte [rsp], 0\n",
                         //"  jne {}"),
                         //loop_label, loop_label);
        byte_code
      }
    }
  }
}

impl SimpleCompiler {
  pub fn new() -> Self {
    SimpleCompiler {
      loop_count: 0,
    }
  }

  fn next_loop_label(&mut self) -> Label {
    let label = Label(format!("LOOP{}", self.loop_count));
    self.loop_count += 1;
    label
  }
}
