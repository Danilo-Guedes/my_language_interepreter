use crate::{
    ast::{ExpressionNode, Program, StatementNode},
    object::Object,
};

const TRUE: Object = Object::Boolean(true);
const FALSE: Object = Object::Boolean(false);
const NULL: Object = Object::Null;

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
            return match expr {
                ExpressionNode::Integer(int) => Object::Integer(int.value),
                ExpressionNode::BooleanNode(boolean) => {
                    Self::native_bool_to_boolean_object(boolean.value)
                }
                ExpressionNode::Prefix(prefix_exp) => {
                    let right: Object = self.eval_expression(Some(*prefix_exp.right));
                    return Self::eval_prefix_expression(prefix_exp.operator, right);
                }
                ExpressionNode::Infix(inf_exp) => {
                    let left: Object = self.eval_expression(Some(*inf_exp.left));
                    let right: Object = self.eval_expression(Some(*inf_exp.right));
                    return Self::eval_infix_expression(inf_exp.operator, &left, &right);
                }
                _ => NULL,
            };
        }
        NULL
    }

    fn native_bool_to_boolean_object(input: bool) -> Object {
        if input {
            TRUE
        } else {
            FALSE
        }
    }

    fn eval_prefix_expression(operator: String, right: Object) -> Object {
        match operator.as_str() {
            "!" => Self::eval_bang_operator_expression(right),
            "-" => Self::eval_minus_prefix_operator_expression(right),
            _ => NULL,
        }
    }

    fn eval_bang_operator_expression(right: Object) -> Object {
        match right {
            Object::Boolean(true) => FALSE,
            Object::Boolean(false) => TRUE,
            Object::Null => TRUE,
            _ => FALSE,
        }
    }

    fn eval_minus_prefix_operator_expression(right: Object) -> Object {
        match right {
            Object::Integer(value) => Object::Integer(-value),
            _ => NULL,
        }
    }

    fn eval_infix_expression(operator: String, left: &Object, right: &Object) -> Object {
        match (left, right) {
            (Object::Integer(left_val), Object::Integer(right_val)) => {
                Self::eval_integer_infix_expression(operator, *left_val, *right_val)
            }
            _ => NULL,
        }
    }

    fn eval_integer_infix_expression(operator: String, left: i64, right: i64) -> Object {
        match operator.as_str() {
            "+" => Object::Integer(left + right),
            "-" => Object::Integer(left - right),
            "*" => Object::Integer(left * right),
            "/" => Object::Integer(left / right),
            _ => NULL,
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
            ("-5", -5),
            ("-10", -10),
            ("5 + 5 + 5 + 5 - 10", 10),
            ("2 * 2 * 2 * 2 * 2", 32),
            ("-50 + 100 + -50", 0),
            ("5 * 2 + 10", 20),
            ("5 + 2 * 10", 25),
            ("20 + 2 * -10", 0),
            ("50 / 2 * 2 + 10", 60),
            ("2 * (5 + 10)", 30),
            ("3 * 3 * 3 + 10", 37),
            ("3 * (3 * 3) + 10", 37),
            ("(5 + 10 * 2 + 15 / 3) * 2 + -10", 50),
        ];

        for test in tests {
            let evaluated = test_eval(test.0);
            test_integer_object(evaluated, test.1);
        }
    }

    #[test]
    fn test_eval_boolean_expression() {
        let tests = vec![("true", true), ("false", false)];

        for test in tests {
            let evaluated = test_eval(test.0);
            test_boolean_object(evaluated, test.1);
        }
    }

    #[test]
    fn test_bang_operator() {
        let tests = vec![
            ("!true", false),
            ("!false", true),
            ("!5", false),
            ("!!true", true),
            ("!!false", false),
            ("!!5", true),
        ];

        for test in tests {
            let evaluated = test_eval(test.0);
            test_boolean_object(evaluated, test.1);
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

    fn test_boolean_object(obj: Object, expected: bool) {
        match obj {
            Object::Boolean(value) => assert_eq!(
                value, expected,
                "object has wrong value, got={} expected={} ",
                value, expected
            ),
            other => panic!("Expected Boolean, got {:?}", other),
        }
    }
}
