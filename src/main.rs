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

    code.parse();
    code.run();
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
