use crate::ast::Node;
use crate::lexer::Lexer;
use crate::parser::Parser;
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

        let lexer: Lexer = Lexer::new(&input);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().expect("Failed to parse program");

        if parser.errors().len() != 0 {
            print_parse_errors(&stdout, parser.errors());
            continue;
        }

        let parsed_program_string = program.print_string();

        writeln!(stdout, "{}", parsed_program_string).expect("Failed to write to stdout");
    }
}

fn print_parse_errors(mut stdout: &Stdout, errors: &Vec<String>) {
    writeln!(stdout, "Oops! We ran into parser errors")
        .expect("Failed to write print_parse_errors to stdout");
    for error in errors {
        writeln!(stdout, "{}", error).expect("Failed to write to stdout");
    }
}
