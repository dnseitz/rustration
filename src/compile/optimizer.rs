// compile/optimizer.rs
// Rustration
//
// Created by Daniel Seitz on 1/12/17

use std::collections::VecDeque;
use super::bytecode::{ByteCode, ByteProgram};

pub struct Optimizer {
  program: ByteProgram,
}

impl Optimizer {
  pub fn new(program: ByteProgram) -> Self {
    Optimizer {
      program: program,
    }
  }

  pub fn optimize(self) -> ByteProgram {
    ByteProgram::from(optimize(self.program.into()))
  }
}

fn optimize(mut byte_code: VecDeque<ByteCode>) -> VecDeque<ByteCode> {
  let mut optimized = VecDeque::with_capacity(byte_code.len());
  while let Some(op) = byte_code.pop_front() {
    match op {
      ByteCode::Add(num) => optimized.append(&mut optimize_add(&mut byte_code, num)),
      ByteCode::Sub(num) => optimized.append(&mut optimize_add(&mut byte_code, -num)),
      ByteCode::MoveRight(num) => optimized.append(&mut optimize_move(&mut byte_code, num)),
      ByteCode::MoveLeft(num) => optimized.append(&mut optimize_move(&mut byte_code, -num)),
      _ => optimized.push_back(op),
    }
  }
  optimized
}

fn optimize_add(byte_code: &mut VecDeque<ByteCode>, mut sum: isize) -> VecDeque<ByteCode> {
  let mut optimized = VecDeque::new();
  while let Some(op) = byte_code.pop_front() {
    match op {
      ByteCode::Add(num) => sum += num,
      ByteCode::Sub(num) => sum -= num,
      _ => {
        if sum > 0 {
          optimized.push_back(ByteCode::Add(sum));
        }
        else if sum < 0 {
          optimized.push_back(ByteCode::Sub(-sum));
        }
        byte_code.push_front(op);
        break;
      }, 
    }
  }
  optimized
}

fn optimize_move(byte_code: &mut VecDeque<ByteCode>, mut sum: isize) -> VecDeque<ByteCode> {
  let mut optimized = VecDeque::new();
  while let Some(op) = byte_code.pop_front() {
    match op {
      ByteCode::MoveRight(num) => sum += num,
      ByteCode::MoveLeft(num) => sum -= num,
      _ => {
        if sum > 0 {
          optimized.push_back(ByteCode::MoveRight(sum));
        }
        else if sum < 0 {
          optimized.push_back(ByteCode::MoveLeft(-sum));
        }
        byte_code.push_front(op);
        break;
      }, 
    }
  }
  optimized
}
