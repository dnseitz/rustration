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

extern crate clap;
use clap::{Arg, App};

use std::fs::File;
use std::path::Path;
use std::io::Read;
use std::process::Command;
use std::error::Error;
use parse::RawParser;
use interpreter::Repl;
use compile::Compiler;
use compile::Optimizer;

mod interpreter;
mod compile;
mod parse;

const GENERAL_ERR: i32 = -1;
const PARSE_ERR: i32 = -2;
//const COMPILE_ERR: i32 = -3;
const ASSEMBLE_ERR: i32 = -4;
const LINK_ERR: i32 = -5;

#[derive(Debug)]
enum Mode {
  Interpret {
    repl: bool,
  },
  Compile {
    optimized: bool,
    no_assemble: bool,
    no_link: bool,
    output_file: String,
  },
}

// For now lets interpret, maybe we can compile in the future...
fn main() {
  // Usage: rustration (-c [-O] [-o output-file] | -i) (input-file | -)
  // -h, --help Help message
  // -O, --optimize Optimize the compiled output
  // -S, --assembly Only run compile steps
  // -c, --no-link Only run compile and assemble steps
  // -o, --out-file Output file
  // -i, --interpret Interpret the file
  //
  let matches = App::new("Rustration")
                        .version("0.1")
                        .author("Daniel Seitz")
                        .about("A command line interpreter/compiler for Brainfuck")
                        .arg(Arg::with_name("optimize")
                             .short("O")
                             .long("optimize")
                             .help("Optimize the compiled output, does nothing if you are running with -i"))
                        .arg(Arg::with_name("assembly")
                             .short("S")
                             .long("assembly")
                             .help("Only run compile steps, does nothing if you are running with -i"))
                        .arg(Arg::with_name("no-link")
                             .short("-c")
                             .long("no-link")
                             .help("Only run compile and assemble steps, does nothgin if you are running with -i"))
                        .arg(Arg::with_name("output")
                             .short("o")
                             .long("out-file")
                             .help("Output file name, does nothing if you are running with -i")
                             .value_name("FILE")
                             .takes_value(true))
                        .arg(Arg::with_name("interpret")
                             .short("i")
                             .long("interpret")
                             .help("Interpret and run the input file without compiling"))
                        .arg(Arg::with_name("INPUT")
                             .help("The input file to use or - for stdin")
                             .required(true))
                        .get_matches();

  let in_file = matches.value_of("INPUT").unwrap();
  let mut in_file_stem = String::from(Path::new(in_file).file_stem().unwrap().to_str().unwrap());
  let no_assemble = matches.is_present("assembly");
  let no_link = matches.is_present("no-link");
  let default_out_file = if no_assemble {
    in_file_stem.push_str(".asm");
    &in_file_stem
  }
  else if no_link {
    in_file_stem.push_str(".o");
    &in_file_stem
  }
  else {
    "a.out"
  };
  let mode = match (matches.is_present("interpret"), in_file) {
    (true, "-") => Mode::Interpret { repl: true },
    (true, _) => Mode::Interpret { repl: false },
    (false, _) => Mode::Compile { 
      optimized: matches.is_present("optimize"),
      no_assemble: matches.is_present("assembly"),
      no_link: matches.is_present("no-link"),
      output_file: String::from(matches.value_of("output").unwrap_or(&default_out_file)),
    },
  };

  match mode {
    Mode::Compile { optimized, output_file, no_assemble, no_link } => {
      let data = match read_file(in_file) {
        Ok(data) => data,
        Err(err) => {
          exit_with_error(GENERAL_ERR, err);
        },
      };
      println!("Compiling with optimization: {}, to output file: {}, from input file: {}", optimized, output_file, in_file);
      // Stage 1: Lex + Parse
      let mut parser = RawParser::new(data);
      match parser.parse() {
        Ok(program) => {
          // Stage 2: Compile to bytecode
          let mut compiler = compile::SimpleCompiler::new();
          let byte_program = compiler.compile_program(&program);

          // Stage 3: Optimize + Emit Assembly
          let asm_path = if no_assemble {
            output_file.clone()
          }
          else {
            get_temp_path("out.asm")
          };
          let mut asm_out = match File::create(&asm_path) {
            Ok(file) => file,
            Err(err) => exit_with_error(GENERAL_ERR, err),
          };

          if !optimized {
            byte_program.emit(&mut asm_out);
          }
          else {
            let optimizer = Optimizer::new(byte_program);
            let optimized = optimizer.optimize();
            optimized.emit(&mut asm_out);
          }

          // Stage 4: Assemble
          if no_assemble {
            cleanup();
            return;
          }

          let asm_out = if no_link {
            Some(output_file.clone())
          }
          else {
            None
          };
          assemble(asm_out);

        
          // Stage 5: Link
          if no_link {
            cleanup();
            return;
          }

          link(&output_file);

          // Tidy up...
          cleanup();
        },
        Err(err) => exit_with_error(PARSE_ERR, err),
      }
    },
    Mode::Interpret { repl: do_repl } => {
      if do_repl {
        let mut repl = Repl::new();
        repl.start();
      }
      else {
        let data = match read_file(in_file) {
          Ok(data) => data,
          Err(err) => exit_with_error(GENERAL_ERR, err),
        };
        let mut code = RawParser::new(data);
        match code.parse() {
          Ok(program) => program.run(),
          Err(err) => exit_with_error(PARSE_ERR, err),
        }
      }
    },
  }
}

fn read_file<P: AsRef<Path>>(path: P) -> std::io::Result<Vec<u8>> {
  let mut file = try!(File::open(path));
  let mut buffer = match file.metadata() {
    Ok(metadata) => {
      let len = metadata.len();
      Vec::with_capacity(len as usize)
    },
    Err(_) => Vec::new(),
  };

  try!(file.read_to_end(&mut buffer));
  Ok(buffer)
}

fn assemble(out_path: Option<String>) {
  let asm_path = get_temp_path("out.asm");
  let obj_path = out_path.unwrap_or(get_temp_path("out.o"));
  let child = Command::new("nasm")
                          .arg("-f")
                          .arg("macho64")
                          .arg(&asm_path)
                          .arg("-o")
                          .arg(&obj_path)
                          .spawn();
  let output = match child {
    Ok(child) => child.wait_with_output(),
    Err(err) => exit_with_error(GENERAL_ERR, err),
  };

  match output {
    Ok(output) => if !output.status.success() {
      let err = if let Ok(s) = String::from_utf8(output.stderr) {
        s
      }
      else {
        String::from("Error executing assembler")
      };
      cleanup();
      println!("{}", err);
      std::process::exit(ASSEMBLE_ERR);
    },
    Err(err) => exit_with_error(1, err),
  }
}

fn link(bin_path: &str) {
  let obj_path = get_temp_path("out.o");
  let child = Command::new("ld")
                         .arg("-lSystem")
                         .arg("-o")
                         .arg(bin_path)
                         .arg(&obj_path)
                         .spawn();
  let output = match child {
    Ok(child) => child.wait_with_output(),
    Err(err) => exit_with_error(1, err),
  };

  match output {
    Ok(output) => if !output.status.success() {
      let err = if let Ok(s) = String::from_utf8(output.stderr) {
        s
      }
      else {
        String::from("Error executing linker")
      };
      cleanup();
      println!("{}", err);
      std::process::exit(LINK_ERR);
    },
    Err(err) => exit_with_error(1, err),
  }
}

fn exit_with_error<E: Error>(code: i32, err: E) -> ! {
  cleanup();
  println!("{}", err);
  std::process::exit(code);
}

fn get_temp_path<P: AsRef<Path>>(path: P) -> String {
  let mut temp_dir = std::env::temp_dir();
  temp_dir.push(path);
  // TODO: Handle this?
  String::from(temp_dir.to_str().unwrap())
}

fn cleanup() {
  let asm_path = get_temp_path("out.asm");
  let obj_path = get_temp_path("out.o");

  std::fs::remove_file(asm_path).ok();
  std::fs::remove_file(obj_path).ok();
}
