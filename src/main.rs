// Copyright 2017 Daniel Seitz.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! A Brainfuck interpreter written in Rust.
//! 
//! ```bf
//! ++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.
//! ```
//! 
//! Brainfuck is an esoteric programming language invented by Urban Muller in 1993. The language
//! operates on an array of memory cells, also called a tape. Every cell is intialized to 0. There
//! is a pointer that intially points to the first memory cell, and several commands are used to
//! manipulate the pointer and the data on the tape. The set of commands are `>`, `<`, `+`, `-`,
//! `[`, `]`, `.`, and `,`. 
//! 
//! `<` and `>` move the data pointer left and right respectively. `+` and `-` increment and
//! decrement the data in the cell being pointed at. `[` and `]` act as a looping mechanism for the
//! language. A `[` command means jump past the matching `]` if the cell under the data pointer is
//! 0. A `]` command means jump back to the matching `[` if the cell under the data pointer is not
//! 0. This looping construct is similar to a while loop in a C-like language. A C representation
//! would be something like:
//! 
//! ```c
//! while *data != 0 {
//! 
//! }
//! ```
//! 
//! The `,` and `.` commands act as input and output respectively. `,` inputs a character and
//! stores it at the data pointer, `.` outputs the character under the data pointer.
//! 
//! All other characters are considered comments and are ignored.
//! 

use std::fs::File;
use std::path::Path;
use std::io::Read;
use std::env;
use parse::Code;
use interpreter::Repl;

mod interpreter;
mod parse;

// For now lets interpret, maybe we can compile in the future...
fn main() {
  if env::args().len() > 1 {
    let file_name = env::args().nth(1).unwrap();
    println!("Filename: {}", &file_name);

    let file = read_file(file_name);

    let mut code = Code::new(file);

    match code.parse() {
    Ok(program) => program.run(),
    Err(err) => println!("{}", err),
    }
  }
  else {
    let mut repl = Repl::new();
    repl.start();
  }
}

fn read_file<P: AsRef<Path>>(path: P) -> Vec<u8> {
  let mut file = File::open(path).unwrap();
  let mut buffer = match file.metadata() {
    Ok(metadata) => {
      let len = metadata.len();
      Vec::with_capacity(len as usize)
    },
    Err(_) => Vec::new(),
  };

  file.read_to_end(&mut buffer).unwrap();
  buffer
}
