use repl::start;
use std::io;

pub mod ast;
pub mod lexer;
pub mod parser;
pub mod repl;
pub mod token;

fn main() {
    println!("\n\nHello!! This is the GuedzLang interpreter!");
    println!("Feel free to type in commands");
    start(io::stdin(), io::stdout())
}
