// compile/bytecode.rs
// Rustration
//
// Created by Daniel Seitz on 1/12/17

use std::collections::VecDeque;
use super::compiler::Label;
use std::io::Write;

#[derive(Debug)]
pub enum ByteCode {
  Add(isize),
  Sub(isize),
  MoveRight(isize),
  MoveLeft(isize),
  Read,
  Write,
  Jump(Label),
  JumpNotZero(Label),
  Exit,
}

#[derive(Debug)]
pub struct ByteProgram {
  program: VecDeque<ByteCode>,
}

impl ByteProgram {
  pub fn new() -> Self {
    ByteProgram {
      program: VecDeque::new(),
    }
  }

  pub fn emit<W: Write>(&self, out: &mut W) {
    // For now write to stdout, in the future we can use a writer
    emit_prelude(out);

    for byte_code in self.program.iter() {
      compile_to_native_code(byte_code, out);
    }

    emit_bss(out);
  }
}

impl From<VecDeque<ByteCode>> for ByteProgram {
  fn from(byte_code: VecDeque<ByteCode>) -> Self {
    ByteProgram {
      program: byte_code,
    }
  }
}

impl From<ByteProgram> for VecDeque<ByteCode> {
  fn from(byte_program: ByteProgram) -> Self {
    byte_program.program
  }
}

fn emit_prelude<W: Write>(out: &mut W) {
  write_all(out, "global start\n");
  write_all(out, "\n");
  write_all(out, "section .text\n");
  write_all(out, "\n");
  write_all(out, "start:\n");
  write_all(out, "  mov rsp, tape\n");
}

fn compile_to_native_code<W: Write>(byte_code: &ByteCode, out: &mut W) {
  match *byte_code {
    ByteCode::Add(num) => { write_all(out, &format!("  add byte [rsp], {}\n", num)); },
    ByteCode::Sub(num) => { write_all(out, &format!("  sub byte [rsp], {}\n", num)); },
    ByteCode::MoveRight(num) => { write_all(out, &format!("  add rsp, {}\n", num)); },
    // TODO: Use rsp by offset, saturating sub for offset reg
    ByteCode::MoveLeft(num) => { write_all(out, &format!("  sub rsp, {}\n", num)); },
    ByteCode::Read => {
      write_all(out, "  mov rax, 0x2000003 ; read\n");
      write_all(out, "  mov rdi, 0         ; stdin\n");
      write_all(out, "  mov rsi, rsp\n");
      write_all(out, "  mov rdx, 1\n");
      write_all(out, "  syscall\n");
    },
    ByteCode::Write => {
      write_all(out, "  mov rax, 0x2000004 ; write\n");
      write_all(out, "  mov rdi, 1         ; stdout\n");
      write_all(out, "  mov rsi, rsp\n");
      write_all(out, "  mov rdx, 1\n");
      write_all(out, "  syscall\n");
    },
    ByteCode::Jump(ref label) => {
      write_all(out, &format!("  jmp _{}\n", label));
      write_all(out, &format!("{}:\n", label));
    },
    ByteCode::JumpNotZero(ref label) => {
      write_all(out, &format!("_{}:\n", label));
      write_all(out, "  cmp byte [rsp], 0\n");
      write_all(out, &format!("  jne {}\n", label));
    },
    ByteCode::Exit => {
      write_all(out, "  mov rax, 0x2000001 ; exit\n");
      write_all(out, "  mov rdi, 0\n");
      write_all(out, "  syscall\n");
    },
  }
}

fn emit_bss<W: Write>(out: &mut W) {
  write_all(out, "section .bss\n");
  write_all(out, "tape: resq 10000\n");
}

fn write_all<W: Write>(out: &mut W, to_write: &str) {
  if let Err(err) = out.write_all(to_write.as_bytes()) {
    // TODO: Handle this
    panic!("{}", err);
  }
}
