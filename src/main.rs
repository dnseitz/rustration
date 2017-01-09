
use std::fs::File;
use std::path::Path;
use std::io::Read;
use std::env;
use parse::Code;

mod interpreter;
mod parse;

// For now lets interpret, maybe we can compile in the future...
fn main() {
  // TODO: error handling
  let file_name = env::args().nth(1).unwrap();
  println!("Filename: {}", &file_name);

  let file = read_file(file_name);

  let mut code = Code::new(file);

  code.parse();
  code.run();

  //println!("{:#?}", code);

  //interpreter::interpret(file);
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
