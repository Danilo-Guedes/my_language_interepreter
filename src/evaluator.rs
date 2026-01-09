use crate::{
    ast::{ExpressionNode, Program, StatementNode},
    object::Object,
};

pub struct Evaluator {}

impl Evaluator {
    pub fn new() -> Self {
        Evaluator {}
    }

    pub fn eval_program(&self, program: Program) -> Object {
        let mut restult = Object::Null;

        for stmt in program.statements {
            restult = self.eval_statement(stmt);
        }
        restult
    }

    fn eval_statement(&self, stmt: StatementNode) -> Object {
        match stmt {
            StatementNode::Expression(exp_stmt) => self.eval_expression(exp_stmt.expression),
            _ => Object::Null,
        }
    }

    fn eval_expression(&self, expression: Option<ExpressionNode>) -> Object {
        if let Some(expr) = expression {
            match expr {
                ExpressionNode::Integer(int) => Object::Integer(int.value),
                _ => Object::Null,
            }
        } else {
            Object::Null
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{lexer::Lexer, object::Object, parser::Parser};

    use super::Evaluator;

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

        let evaluator = Evaluator::new();
        evaluator.eval_program(program)
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
