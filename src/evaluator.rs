use crate::{ast::Program, object::Object};

fn eval_program(program: Program) -> Object {
    todo!();
    // let mut result = Object::Null;

    // for statement in program.statements {
    //     result = eval_statement(statement);
    // }

    // result
}

#[cfg(test)]
mod test {
    use crate::{lexer::Lexer, object::Object, parser::Parser};

    use super::eval_program;

    #[test]
    fn test_eval_integer_expression() {
        let tests = vec![
            ("5", 5),
            ("10", 10),
            // ("-5", -5),
            // ("-10", -10),
            // ("5 + 5 + 5 + 5 - 10", 10),
            // ("2 * 2 * 2 * 2 * 2", 32),
            // ("-50 + 100 + -50", 0),
            // ("5 * 2 + 10", 20),
            // ("5 + 2 * 10", 25),
            // ("20 + 2 * -10", 0),
            // ("50 / 2 * 2 + 10", 60),
            // ("2 * (5 + 10)", 30),
            // ("3 * 3 * 3 + 10", 37),
            // ("3 * (3 * 3) + 10", 37),
            // ("(5 + 10 * 2 + 15 / 3) * 2 + -10", 50),
        ];

        for test in tests {
            let evaluated = test_eval(test.0);
            test_integer_object(evaluated, test.1);
        }
    }

    fn test_eval(input: &str) -> Object {
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().expect("Failed to parse program");
        eval_program(program)
    }

    fn test_integer_object(obj: Object, expected: i64) {
        match obj {
            Object::Integer(value) => assert_eq!(
                value, expected,
                "object has wrong value, got={} expected={} ",
                value, expected
            ),
            other => panic!("Expected Integer, got {:?}", other),
        }
    }
}
