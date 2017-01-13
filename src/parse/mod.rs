
//! Parsing Brainfuck programs.
//! 
//! Because lexing is so simple for this language, the lexer runs lazily, only lexing the input as
//! a new token is required. This means the lexing and parsing stages are very closely linked as
//! the parser and lexer run essentially in lockstep.
//! 
//! The parser constructs an AST of the Brainfuck program. Each program can be represented by one
//! single block statement that contains all the expressions in the program stored sequentially.
//! 
//! Because each expression is so simple, code can easily be executed at parse time. This means
//! that as the program is being parsed it can be executed on a virtual machine. This is what
//! allows the REPL functionality of this interpreter.

pub mod ast;
mod token;
mod error;
mod parsing;

pub use self::token::EOF;
pub use self::parsing::{ReplParser, RawParser};
use std;

pub type Result<T> = std::result::Result<T, error::ParseError>;
