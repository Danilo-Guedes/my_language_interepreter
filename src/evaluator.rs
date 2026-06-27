use std::{collections::HashMap, ops::Deref};

use crate::{
    ast::{BlockStatement, ExpressionNode, Identifier, IfExpression, Program, StatementNode},
    object::{
        Env, Environment, Function, HashPair, HashStruct, Hashable, Object, FALSE, NULL, TRUE,
    },
};

pub struct Evaluator {
    env: Env,
}

impl Evaluator {
    pub fn new() -> Self {
        Evaluator {
            env: Environment::new_environment(),
        }
    }

    pub fn eval_program(&mut self, program: Program) -> Object {
        let mut result = Object::Null;

        for stmt in program.statements {
            result = self.eval_statement(stmt);

            if let Object::ReturnValue(ret) = result {
                return *ret;
            }
            if let Object::Error(_) = result {
                return result;
            }
        }
        result
    }

    fn eval_statement(&mut self, stmt: StatementNode) -> Object {
        match stmt {
            StatementNode::Expression(exp_stmt) => self.eval_expression(exp_stmt.expression),
            StatementNode::Return(ret_stmt) => {
                let value = self.eval_expression(ret_stmt.return_value);
                if Self::is_error(&value) {
                    return value;
                }
                Object::ReturnValue(Box::new(value))
            }
            StatementNode::Let(let_stmt) => {
                let value = self.eval_expression(let_stmt.value);
                if Self::is_error(&value) {
                    return value;
                }
                self.env
                    .borrow_mut()
                    .set(let_stmt.name.value, value.clone());
                value
            }
            _ => Object::Null,
        }
    }

    fn eval_expression(&mut self, expression: ExpressionNode) -> Object {
        match expression {
            ExpressionNode::Integer(int) => Object::Integer(int.value),
            ExpressionNode::BooleanNode(boolean) => {
                Self::native_bool_to_boolean_object(boolean.value)
            }
            ExpressionNode::Prefix(prefix_exp) => {
                let right: Object = self.eval_expression(*prefix_exp.right);
                if Self::is_error(&right) {
                    return right;
                }
                Self::eval_prefix_expression(prefix_exp.operator, right)
            }
            ExpressionNode::Infix(inf_exp) => {
                let left: Object = self.eval_expression(*inf_exp.left);
                if Self::is_error(&left) {
                    return left;
                }
                let right: Object = self.eval_expression(*inf_exp.right);
                if Self::is_error(&right) {
                    return right;
                }
                Self::eval_infix_expression(inf_exp.operator, &left, &right)
            }
            ExpressionNode::IfExpressionNode(if_exp) => self.eval_if_expression(if_exp),
            ExpressionNode::IdentifierNode(ident) => self.eval_identifier(ident),
            ExpressionNode::Function(fn_lit) => Object::Func(Function {
                parameters: fn_lit.parameters,
                body: fn_lit.body,
                env: self.env.clone(),
            }),
            ExpressionNode::Call(call_exp) => {
                let function = self.eval_expression(call_exp.function.deref().clone());
                if Self::is_error(&function) {
                    return function;
                }
                let args = self.eval_expressions(call_exp.arguments);

                if args.len() == 1 && Self::is_error(&args[0]) {
                    return args[0].clone();
                }

                self.apply_function(function, args)
            }
            ExpressionNode::StringExp(string_literal) => Object::StringObj(string_literal.value),
            ExpressionNode::Array(array_literal) => {
                let elements = self.eval_expressions(array_literal.elements);

                if elements.len() == 1 && Self::is_error(&elements[0]) {
                    return elements[0].clone();
                }
                Object::Array(elements)
            }
            ExpressionNode::Index(index_exp) => {
                let left = self.eval_expression(*index_exp.left);
                if Self::is_error(&left) {
                    return left;
                }

                let index = self.eval_expression(*index_exp.index);
                if Self::is_error(&index) {
                    return index;
                }

                self.eval_index_expression(left, index)
            }
            ExpressionNode::Hash(hash_literal) => {
                let mut pairs = HashMap::new();

                for (key_node, value_node) in hash_literal.pairs {
                    let key = self.eval_expression(key_node);
                    if Self::is_error(&key) {
                        return key;
                    }
                    let value = self.eval_expression(value_node);
                    if Self::is_error(&value) {
                        return value;
                    }
                    let hash_key = match key.hash_key() {
                        Ok(hash_key) => hash_key,
                        Err(err) => return Object::Error(err),
                    };
                    pairs.insert(hash_key, HashPair { key, value });
                }
                Object::HashObj(HashStruct { pairs })
            }
            _ => NULL,
        }
    }

    fn eval_index_expression(&self, left: Object, index: Object) -> Object {
        match (&left, &index) {
            (Object::Array(_), Object::Integer(_)) => {
                Self::eval_array_index_expression(left, index)
            }
            (Object::HashObj(_), _) => Self::eval_hash_index_expression(left, index),
            _ => Object::Error(format!(
                "index operator not supported: {}",
                left.object_type()
            )),
        }
    }

    fn eval_hash_index_expression(hash: Object, index: Object) -> Object {
        match hash {
            Object::HashObj(hash_struct) => {
                let key = match index.hash_key() {
                    Ok(hash_key) => hash_key,
                    Err(err) => return Object::Error(err),
                };

                let pair = match hash_struct.pairs.get(&key) {
                    Some(hash_pair) => hash_pair,
                    None => return NULL,
                };

                pair.value.clone()
            }

            _ => Object::Error(format!(
                "index operator not supported: {}",
                hash.object_type()
            )),
        }
    }

    fn eval_array_index_expression(array: Object, index: Object) -> Object {
        if let Object::Array(arr) = array {
            if let Object::Integer(idx) = index {
                if arr.is_empty() {
                    return NULL;
                }
                let max = (arr.len() - 1) as i64;
                if idx < 0 || idx > max {
                    return NULL;
                }
                return arr[idx as usize].clone();
            }
        }
        NULL
    }

    fn apply_function(&mut self, func: Object, args: Vec<Object>) -> Object {
        match func {
            Object::Func(function) => {
                let old_env = self.env.clone();
                let extended_env = self.extended_function_env(function.clone(), args);

                self.env = extended_env;
                let evaluated = self.eval_block_statement(function.body);
                self.env = old_env;
                Self::unwrap_return_value(evaluated)
            }
            Object::Builtin(b_fn) => b_fn(args),
            _ => Object::Error(format!("not a function: {}", func.object_type())),
        }
    }

    fn extended_function_env(&self, function: Function, args: Vec<Object>) -> Env {
        let env = Environment::new_enclosed_environment(function.env);

        for (idx, param) in function.parameters.into_iter().enumerate() {
            env.borrow_mut().set(param.value, args[idx].clone());
        }
        env
    }

    fn unwrap_return_value(obj: Object) -> Object {
        match obj {
            Object::ReturnValue(ret) => *ret,
            _ => obj,
        }
    }

    fn eval_expressions(&mut self, expressions: Vec<ExpressionNode>) -> Vec<Object> {
        let mut result = Vec::new();

        for exp in expressions {
            let evaluated = self.eval_expression(exp);
            if Self::is_error(&evaluated) {
                return vec![evaluated];
            }
            result.push(evaluated);
        }
        result
    }
    fn native_bool_to_boolean_object(input: bool) -> Object {
        if input {
            TRUE
        } else {
            FALSE
        }
    }

    fn is_error(obj: &Object) -> bool {
        matches!(obj, Object::Error(_))
    }

    fn eval_prefix_expression(operator: String, right: Object) -> Object {
        match operator.as_str() {
            "!" => Self::eval_bang_operator_expression(right),
            "-" => Self::eval_minus_prefix_operator_expression(right),
            _ => Object::Error(format!(
                "unknown operator: {} {}",
                operator,
                right.object_type()
            )),
        }
    }

    fn eval_infix_expression(operator: String, left: &Object, right: &Object) -> Object {
        if left.object_type() != right.object_type() {
            return Object::Error(format!(
                "type mismatch: {} {} {}",
                left.object_type(),
                operator,
                right.object_type()
            ));
        };
        match (left, right, operator) {
            (Object::Integer(left_val), Object::Integer(right_val), op) => {
                Self::eval_integer_infix_expression(op, *left_val, *right_val)
            }
            (Object::Boolean(left_val), Object::Boolean(right_val), op) => match op.as_str() {
                "==" => Self::native_bool_to_boolean_object(left_val == right_val),
                "!=" => Self::native_bool_to_boolean_object(left_val != right_val),
                _ => Object::Error(format!(
                    "unknown operator: {} {} {}",
                    left.object_type(),
                    op,
                    right.object_type()
                )),
            },
            (Object::StringObj(left_str), Object::StringObj(right_str), op) => match op.as_str() {
                "+" => Object::StringObj(format!("{}{}", left_str, right_str)),
                _ => Object::Error(format!(
                    "unknown operator: {} {} {}",
                    left.object_type(),
                    op,
                    right.object_type()
                )),
            },
            (left, right, op) => Object::Error(format!(
                "unknown operator: {} {} {}",
                left.object_type(),
                op,
                right.object_type()
            )),
        }
    }

    fn eval_if_expression(&mut self, if_exp: IfExpression) -> Object {
        let condition = self.eval_expression(*if_exp.condition);

        if Self::is_truthy(condition) {
            self.eval_block_statement(if_exp.consequence)
        } else if let Some(alternative) = if_exp.alternative {
            self.eval_block_statement(alternative)
        } else {
            NULL
        }
    }

    fn is_truthy(obj: Object) -> bool {
        match obj {
            Object::Null => false,
            Object::Boolean(true) => true,
            Object::Boolean(false) => false,
            _ => true,
        }
    }

    fn eval_identifier(&self, identifier: Identifier) -> Object {
        let value = self.env.borrow().get(&identifier.value);
        match value {
            Some(val) => val,
            None => Object::Error(format!("identifier not found: {}", identifier.value)),
        }
    }

    fn eval_block_statement(&mut self, block: BlockStatement) -> Object {
        let mut result = Object::Null;

        for stmt in block.statements {
            result = self.eval_statement(stmt);

            if matches!(result, Object::ReturnValue(_) | Object::Error(_)) {
                return result;
            }
        }
        result
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
            _ => Object::Error(format!("unknown operator: -{}", right.object_type())),
        }
    }

    fn eval_integer_infix_expression(operator: String, left: i64, right: i64) -> Object {
        match operator.as_str() {
            "+" => Object::Integer(left + right),
            "-" => Object::Integer(left - right),
            "*" => Object::Integer(left * right),
            "/" => Object::Integer(left / right),
            "<" => Self::native_bool_to_boolean_object(left < right),
            ">" => Self::native_bool_to_boolean_object(left > right),
            "==" => Self::native_bool_to_boolean_object(left == right),
            "!=" => Self::native_bool_to_boolean_object(left != right),
            _ => NULL,
        }
    }
}

impl Default for Evaluator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {
    use std::any;

    use crate::{
        lexer::Lexer,
        object::{Hashable, Object, FALSE, NULL, TRUE},
        parser::Parser,
    };

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
        let tests = vec![
            ("true", true),
            ("false", false),
            ("1 < 2", true),
            ("1 > 2", false),
            ("1 > 1", false),
            ("1 == 1", true),
            ("1 != 1", false),
            ("1 == 2", false),
            ("1 != 2", true),
            ("true == true", true),
            ("false == false", true),
            ("true != false", true),
            ("false != true", true),
            ("(1 < 2) == true", true),
            ("(1 < 2) == false", false),
            ("(1 > 2) == true", false),
            ("(1 > 2) == false", true),
        ];

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

    #[test]
    fn test_id_else_expression() {
        let tests = vec![
            ("if (true) { 10 }", 10),
            ("if (false) { 10 }", -999),
            ("if (1) { 10 }", 10),
            ("if (1 < 2) { 10 }", 10),
            ("if (1 > 2) { 10 }", -999),
            ("if (1 > 2) { 10 } else { 20 }", 20),
            ("if (1 < 2) { 10 } else { 20 }", 10),
        ];

        for test in tests {
            let evaluated = test_eval(test.0);
            if test.1 == -999 {
                test_null_object(evaluated);
            } else {
                test_integer_object(evaluated, test.1);
            }
        }
    }

    #[test]
    fn test_return_statements() {
        let tests = vec![
            ("return 10;", 10),
            ("return 10; 9;", 10),
            ("return 2 * 5; 9;", 10),
            ("9; return 2 * 5; 9;", 10),
            ("if (10 > 1) { if (10 > 1) { return 10; } return 1; }", 10),
        ];

        for test in tests {
            let evaluated = test_eval(test.0);
            test_integer_object(evaluated, test.1);
        }
    }

    #[test]
    fn test_error_handling() {
        let tests = vec![
            ("5 + true;", "type mismatch: INTEGER + BOOLEAN"),
            ("5 + true; 5;", "type mismatch: INTEGER + BOOLEAN"),
            ("-true", "unknown operator: -BOOLEAN"),
            ("true + false;", "unknown operator: BOOLEAN + BOOLEAN"),
            ("5; true + false; 5", "unknown operator: BOOLEAN + BOOLEAN"),
            (
                "if (10 > 1) { true + false; }",
                "unknown operator: BOOLEAN + BOOLEAN",
            ),
            (
                "if (10 > 1 ) {
                if (10 > 1 ) {
                return true + false;
                    }
                    return 1;
            ",
                "unknown operator: BOOLEAN + BOOLEAN",
            ),
            ("foobar", "identifier not found: foobar"),
            (r#""Hello" - "World""#, "unknown operator: STRING - STRING"),
            (
                r#"{"name": "Monkey"}[fn(x) { x }];"#,
                "unusable as hash key: FUNCTION",
            ),
        ];

        for test in tests {
            let evaluated = test_eval(test.0);
            match evaluated {
                Object::Error(err) => assert_eq!(err, test.1),
                _ => panic!("Expected error object, got {:?}", evaluated),
            }
        }
    }

    #[test]
    fn test_let_statements() {
        let tests = vec![
            ("let a = 5; a;", 5),
            ("let a = 5 * 5; a;", 25),
            ("let a = 5; let b = a; b;", 5),
            ("let a = 5; let b = a; let c = a + b + 5; c;", 15),
        ];

        for test in tests {
            test_integer_object(test_eval(test.0), test.1);
        }
    }

    #[test]
    fn test_function_object() {
        let input = "fn(x) { x + 2; };";
        let evaluated = test_eval(input);
        match evaluated {
            Object::Func(func) => {
                assert_eq!(
                    func.parameters.len(),
                    1,
                    "function has wrong parameters length. got={}",
                    func.parameters.len()
                );
                assert_eq!(
                    func.parameters[0].to_string(),
                    "x",
                    "paramentes is not 'x', got={}",
                    func.parameters[0].to_string()
                );
                assert_eq!(
                    func.body.to_string(),
                    "(x + 2)",
                    "function body is not '(x + 2);', got={}",
                    func.body.to_string()
                );
            }
            _ => panic!("object is not Function, got {:?}", evaluated),
        }
    }

    #[test]
    fn test_function_application() {
        let tests = vec![
            ("let identity = fn(x) { x; }; identity(5);", 5),
            ("let identity = fn(x) { return x; }; identity(5);", 5),
            ("let double = fn(x) { x * 2; }; double(5);", 10),
            ("let add = fn(x, y) { x + y; }; add(5, 5);", 10),
            ("let add = fn(x, y) { x + y; }; add(5 + 5, add(5, 5));", 20),
            ("fn(x) { x; }(5);", 5),
        ];

        for test in tests {
            test_integer_object(test_eval(test.0), test.1)
        }
    }

    #[test]
    fn test_closures() {
        let input = r#"
        let newAdder = fn(x) {
            fn(y) { x + y };
        };

        let addTwo = newAdder(2);

        addTwo(2);
        "#;

        test_integer_object(test_eval(input), 4);
    }

    #[test]
    fn test_string_literal() {
        let input = r#""Hello World!""#;
        let evaluated = test_eval(input);
        match evaluated {
            Object::StringObj(str) => {
                assert_eq!(str, "Hello World!", "String has wrong value, got={}", str)
            }
            other => panic!("object is not String, got {:?}", other),
        }
    }

    #[test]
    fn test_string_concatenation() {
        let input = r#""Hello" + " " + "World!""#;
        let evaluated = test_eval(input);
        match evaluated {
            Object::StringObj(str) => {
                assert_eq!(str, "Hello World!", "String has wrong value, got={}", str)
            }
            other => panic!("object is not String, got {:?}", other),
        }
    }

    #[test]
    fn test_builtin_functions() {
        let tests: Vec<(&str, Box<dyn any::Any>)> = vec![
            (r#"len("")"#, Box::new(0_i64)),
            (r#"len("four")"#, Box::new(4_i64)),
            (r#"len("hello world")"#, Box::new(11_i64)),
            (
                r#"len(1)"#,
                Box::new(String::from("argument to `len` not supported, got INTEGER")),
            ),
            (
                r#"len("one", "two")"#,
                Box::new(String::from("wrong number of arguments. got=2, want=1")),
            ),
            (r#"len([1, 2, 3])"#, Box::new(3_i64)),
            (r#"first([1, 2, 3])"#, Box::new(1_i64)),
            (r#"last([1, 2, 3])"#, Box::new(3_i64)),
            (r#"rest([1, 2, 3])"#, Box::new(vec![2_i64, 3_i64])),
            (
                r#"push([1, 2, 3], 4)"#,
                Box::new(vec![1_i64, 2_i64, 3_i64, 4_i64]),
            ),
        ];

        for test in tests {
            let evaluated = test_eval(test.0);

            match test.1.downcast_ref::<i64>() {
                Some(expected) => test_integer_object(evaluated, *expected),
                None => match test.1.downcast_ref::<String>() {
                    Some(expected) => match evaluated {
                        Object::Error(err) => assert_eq!(err, *expected),

                        other => panic!("object is not Error, got {:?}", other),
                    },
                    None => match test.1.downcast_ref::<Vec<i64>>() {
                        Some(expected) => match evaluated {
                            Object::Array(arr) => {
                                let arr_values: Vec<i64> = arr
                                    .into_iter()
                                    .map(|obj| match obj {
                                        Object::Integer(value) => value,
                                        _ => panic!("array element is not Integer, got {:?}", obj),
                                    })
                                    .collect();
                                assert_eq!(arr_values, *expected);
                            }
                            other => panic!("object is not Array, got {:?}", other),
                        },
                        None => panic!("unsupported test type"),
                    },
                },
            }
        }
    }

    #[test]
    fn test_array_literals() {
        let input = "[1, 2 * 2, 3 + 3]";
        let evaluated = test_eval(input);
        match evaluated {
            Object::Array(elements) => {
                assert_eq!(
                    elements.len(),
                    3,
                    "array has wrong length, got={}",
                    elements.len()
                );
                test_integer_object(elements[0].clone(), 1);
                test_integer_object(elements[1].clone(), 4);
                test_integer_object(elements[2].clone(), 6);
            }
            other => panic!("object is not Array, got {:?}", other),
        }
    }

    #[test]
    fn test_array_index_expressions() {
        let tests: Vec<(&str, Box<dyn any::Any>)> = vec![
            ("[1, 2, 3][0]", Box::new(1_i64)),
            ("[1, 2, 3][1]", Box::new(2_i64)),
            ("[1, 2, 3][2]", Box::new(3_i64)),
            ("let i = 0; [1][i];", Box::new(1_i64)),
            ("[1, 2, 3][1 + 1];", Box::new(3_i64)),
            ("let myArray = [1, 2, 3]; myArray[2];", Box::new(3_i64)),
            (
                "let myArray = [1, 2, 3]; myArray[0] + myArray[1] + myArray[2] ;",
                Box::new(6_i64),
            ),
            (
                "let myArray = [1, 2, 3]; let i = myArray[0]; myArray[i]",
                Box::new(2_i64),
            ),
            ("[1, 2, 3][3]", Box::new(NULL)), // HERE MOST LANGUAGES WOULD ERROR, WE CHOOSE TO RETURN NULL BY DESIGN SIMPLICITY
            ("[1, 2, 3][-1]", Box::new(NULL)),
        ];

        for test in tests {
            let evaluated = test_eval(test.0);
            match test.1.downcast_ref::<i64>() {
                Some(expected) => test_integer_object(evaluated, *expected),
                None => test_null_object(evaluated),
            }
        }
    }

    #[test]
    fn test_hash_literals() {
        let input = r#"let two = "two";
        {
        "one": 10 -9,
        "two": 1 + 1,
        "thr" + "ee": 6/2,
        4: 4,
        true: 5,
        false: 6,
        }
            
            "#;

        let evaluated = test_eval(input);

        match evaluated {
            Object::HashObj(hash) => {
                let expected = vec![
                    (Object::StringObj("one".to_string()).hash_key(), 1),
                    (Object::StringObj("two".to_string()).hash_key(), 2),
                    (Object::StringObj("three".to_string()).hash_key(), 3),
                    (Object::Integer(4).hash_key(), 4),
                    (TRUE.hash_key(), 5),
                    (FALSE.hash_key(), 6),
                ];

                assert_eq!(
                    hash.pairs.len(),
                    expected.len(),
                    "hash object has wrong number of pairs. got={}, expected={}",
                    hash.pairs.len(),
                    expected.len()
                );

                for (expected_key, expected_value) in expected {
                    let pair = match hash.pairs.get(&expected_key.unwrap()) {
                        Some(hash_pair) => hash_pair,
                        None => panic!("no pair for given key in Pairs"),
                    };
                    test_integer_object(pair.value.clone(), expected_value);
                }
            }
            other => panic!("object is not Hash, got {:?}", other),
        }
    }

    #[test]
    fn test_hash_index_expressions() {
        let tests: Vec<(&str, Box<dyn any::Any>)> = vec![
            (r#"{"foo": 5}["foo"]"#, Box::new(5_i64)),
            (r#"{"foo": 5}["bar"]"#, Box::new(NULL)),
            (r#"let key = "foo"; {"foo": 5}[key]"#, Box::new(5_i64)),
            (r#"{}["foo"]"#, Box::new(NULL)),
            (r#"{5: 5}[5]"#, Box::new(5_i64)),
            (r#"{true: 5}[true]"#, Box::new(5_i64)),
            (r#"{false: 5}[false]"#, Box::new(5_i64)),
        ];

        for test in tests {
            let evaluated = test_eval(test.0);
            match test.1.downcast_ref::<i64>() {
                Some(expected) => test_integer_object(evaluated, *expected),
                None => test_null_object(evaluated),
            }
        }
    }

    #[test]
    fn test_recursive_function() {
        // Regression: a `let`-bound function must be able to call itself.
        // Currently fails ("identifier not found: fib") because the closure
        // captures an environment SNAPSHOT taken before the `let` binding is
        // inserted, so the function's own name is not in its captured scope.
        let input =
            "let fib = fn(x) { if (x < 2) { x } else { fib(x - 1) + fib(x - 2) } }; fib(10);";
        test_integer_object(test_eval(input), 55);
    }

    #[test]
    fn test_empty_array_index() {
        // Regression: indexing an empty array must return NULL, not panic.
        // Currently panics with "attempt to subtract with overflow" because
        // `arr.len() - 1` underflows (usize) before the bounds check runs.
        test_null_object(test_eval("[][0]"));
    }

    fn test_null_object(obj: Object) {
        match obj {
            Object::Null => assert!(true),
            _ => assert!(false),
        }
    }

    fn test_eval(input: &str) -> Object {
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().expect("Failed to parse program");

        let mut evaluator = Evaluator::new();
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
