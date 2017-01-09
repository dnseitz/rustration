
use std::fs::File;
use std::path::Path;
use std::io::Read;
use std::env;
use token::Token;

mod token;
mod interpreter;

// For now lets interpret, maybe we can compile in the future...
fn main() {
  // TODO: error handling
  let file_name = env::args().nth(1).unwrap();
  println!("Filename: {}", &file_name);

  let file = read_file(file_name);

  interpreter::interpret(file);
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

fn parse(stream: Vec<u8>) {
  for token in stream {
    println!("{:#?}", Token::from(token));
  }
}
