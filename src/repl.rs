use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::{evaluator::Evaluator, object::Object};
use std::io::{Stdin, Stdout, Write};

pub fn start(stdin: Stdin, mut stdout: Stdout) -> std::io::Result<()> {
    let mut evaluator = Evaluator::new();

    loop {
        write!(stdout, ">> ")?;
        stdout.flush()?;

        let mut input = String::new();

        let bytes_read = stdin.read_line(&mut input);

        match bytes_read {
            Ok(0) => {
                writeln!(stdout, "Exiting REPL...")?;
                return Ok(());
            }
            Ok(_) => {
                // Successfully read input, continue with processing
            }
            Err(e) => {
                writeln!(stdout, "Failed to read from stdin: {}", e)?;
                return Err(e);
            }
        }

        let lexer: Lexer = Lexer::new(&input);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().expect("Failed to parse program");

        if !parser.errors().is_empty() {
            print_parse_errors(&stdout, parser.errors())?;
            continue;
        }

        let evaluated = evaluator.eval_program(program);

        match &evaluated {
            Object::StringObj(s) => writeln!(stdout, "'{}'", s)?,
            _ => writeln!(stdout, "{}", evaluated)?,
        }
    }
}

fn print_parse_errors(mut stdout: &Stdout, errors: &[String]) -> std::io::Result<()> {
    writeln!(stdout, "Oops! We ran into parser errors")?;
    for error in errors {
        writeln!(stdout, "{}", error)?;
    }
    Ok(())
}
