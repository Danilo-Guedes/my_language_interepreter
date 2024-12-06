use repl::start;
use std::io;

pub mod lexer;
pub mod repl;
pub mod token;
pub mod ast;
pub mod parser;

fn main() {
    println!("\n\nHello!! This is the GuedzLang interpreter!");
    println!("Feel free to type in commands");
    start(io::stdin(), io::stdout())
}
