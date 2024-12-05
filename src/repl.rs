use crate::lexer::Lexer;
use crate::token::TokenKind;
use std::io::{Stdin, Stdout, Write};

pub fn start(stdin: Stdin, mut stdout: Stdout) {
    loop {
        write!(stdout, ">> ").expect("Failed to write to stdout");
        stdout.flush().expect("Failed to flush stdout");

        let mut input = String::new();

        if let Err(e) = stdin.read_line(&mut input) {
            writeln!(stdout, "Failed to read from stdin: {}", e)
                .expect("Failed to write to stdout");
            return;
        }

        let mut lexer: Lexer = Lexer::new(&input);
        loop {
            let token = lexer.next_token();
            if token.kind == TokenKind::EOF {
                break;
            }
            writeln!(stdout, "{:?}", token).expect("Failed to write to stdout");
        }
    }
}
