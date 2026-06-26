use repl::start;
use std::io;

pub mod ast;
pub mod builtins;
pub mod evaluator;
pub mod lexer;
pub mod object;
pub mod parser;
pub mod repl;
pub mod token;

fn main() -> std::io::Result<()> {
    println!("\n\nHello!! This is the GuedzLang REPL...");
    println!("Feel free to type in commands");
    start(io::stdin(), io::stdout())
}
