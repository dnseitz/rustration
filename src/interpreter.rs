
use std::io::{Read, Write};
use token::Token;
use std;

pub fn interpret(stream: Vec<u8>) {
  let mut tape: Vec<isize> = vec![0; 1];
  let mut current_index = 0;
  
  let mut pc = 0;
  let input_len = stream.len();

  'interpreter: while pc < input_len {
    let token = Token::from(stream[pc]);
    match token {
      Token::MoveRight => {
        current_index += 1;
        while current_index >= tape.len() {
          tape.push(0);
        }
      },
      Token::MoveLeft => {
        if current_index != 0 {
          current_index -= 1;
        }
      },
      Token::Increment => {
        tape[current_index] += 1;
      },
      Token::Decrement => tape[current_index] -= 1,
      Token::Output => print!("{}", char::from(tape[current_index] as u8)),
      Token::Input => tape[current_index] = read_input(),
      Token::JumpForward => {
        let mut nesting = 0;
        let mut search_pc = pc + 1;
        while search_pc < input_len {
          let search_token = Token::from(stream[search_pc]);
          if search_token == Token::JumpForward {
            nesting += 1;
          }
          if search_token == Token::JumpBack {
            if nesting > 0 {
              nesting -= 1;
            }
            else {
              pc = if tape[current_index] == 0 {
                search_pc + 1
              }
              else {
                pc + 1
              };
              continue 'interpreter;
            }
          }
          search_pc += 1;
        }
        // TODO: print char number/line number
        panic!("No matching ']' found for '['!");
      },
      Token::JumpBack => {
        let mut nesting = 0;
        let mut search_pc = pc - 1;
        loop {
          if search_pc == 0 {
            break;
          }
          search_pc -= 1;
          let search_token = Token::from(stream[search_pc]);
          if search_token == Token::JumpBack {
            nesting += 1;
          }
          if search_token == Token::JumpForward {
            if nesting > 0 {
              nesting -= 1;
            }
            else {
              pc = if tape[current_index] != 0 {
                search_pc
              }
              else {
                pc + 1
              };
              continue 'interpreter;
            }
          }
        }
        // TODO: print char number/line number
        panic!("No matching '[' found for ']'!");
      },
      Token::Comment => { /* no-op */ },
    }
    pc += 1;
  }
}

fn read_input() -> isize {
  std::io::stdin()
    .bytes()
    .next()
    .and_then(|result| result.ok())
    .map(|byte| byte as isize)
    .unwrap()
}
