use guedzlang::repl::start;
use std::io;

fn main() -> std::io::Result<()> {
    println!("\n\nHello!! This is the GuedzLang REPL...");
    println!("Feel free to type in commands");
    start(io::stdin(), io::stdout())
}
