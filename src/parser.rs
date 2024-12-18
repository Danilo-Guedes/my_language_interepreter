use std::collections::HashMap;

use crate::ast::{
    ExpressionNode, ExpressionStatement, Identifier, InfixExpression, IntegerLiteral, LetStatement,
    PrefixExpression, Program, ReturnStatement, StatementNode,
};
use crate::lexer::Lexer;
use crate::token::{Token, TokenKind};

type PrefixParseFn = fn(&mut Parser) -> Option<ExpressionNode>;
type InfixParseFn = fn(&mut Parser, ExpressionNode) -> Option<ExpressionNode>;

#[derive(Debug, Copy, Clone)]
enum PrecedenceLevel {
    Lowest = 0,
    Equals = 1,      // ==
    LessGreater = 2, // > or <
    Sum = 3,         // +
    Product = 4,
    Prefix = 5,
    Call = 6,
}
fn precedence_map(token_kind: &TokenKind) -> PrecedenceLevel {
    match token_kind {
        TokenKind::EQ | TokenKind::NotEQ => PrecedenceLevel::Equals,
        TokenKind::LT | TokenKind::GT => PrecedenceLevel::LessGreater,
        TokenKind::Plus | TokenKind::Minus => PrecedenceLevel::Sum,
        TokenKind::Slash | TokenKind::Asterisk => PrecedenceLevel::Product,
        TokenKind::LParen => PrecedenceLevel::Call,
        _ => PrecedenceLevel::Lowest,
    }
}

pub struct Parser {
    lexer: Lexer,
    pub cur_token: Token,
    pub peek_token: Token,
    errors: Vec<String>,
    prefix_parse_fns: HashMap<TokenKind, PrefixParseFn>,
    infix_parse_fns: HashMap<TokenKind, InfixParseFn>,
}

impl Parser {
    fn new(lexer: Lexer) -> Parser {
        let mut parser = Parser {
            lexer,
            cur_token: Default::default(),
            peek_token: Default::default(),
            errors: Vec::new(),
            prefix_parse_fns: HashMap::new(),
            infix_parse_fns: HashMap::new(),
        };

        parser.register_prefix(TokenKind::Ident, Self::parse_identifier);
        parser.register_prefix(TokenKind::Int, Self::parse_integer_literal);
        parser.register_prefix(TokenKind::Bang, Self::parse_prefix_expression);
        parser.register_prefix(TokenKind::Minus, Self::parse_prefix_expression);
        parser.register_infix(TokenKind::Plus, Self::parse_infix_expression);
        parser.register_infix(TokenKind::Minus, Self::parse_infix_expression);
        parser.register_infix(TokenKind::Slash, Self::parse_infix_expression);
        parser.register_infix(TokenKind::Asterisk, Self::parse_infix_expression);
        parser.register_infix(TokenKind::EQ, Self::parse_infix_expression);
        parser.register_infix(TokenKind::NotEQ, Self::parse_infix_expression);
        parser.register_infix(TokenKind::LT, Self::parse_infix_expression);
        parser.register_infix(TokenKind::GT, Self::parse_infix_expression);

        parser.next_token();
        parser.next_token();

        return parser;
    }

    fn next_token(&mut self) {
        self.cur_token = self.peek_token.clone();
        self.peek_token = self.lexer.next_token();
    }

    pub fn parse_program(&mut self) -> Option<Program> {
        let mut program = Program {
            statements: Vec::new(),
        };

        while !self.cur_token_is(TokenKind::EOF) {
            if let Some(stmt) = self.parse_statement() {
                program.statements.push(stmt);
            }
            self.next_token();
        }

        return Some(program);
    }

    fn expect_peek(&mut self, token_kind: TokenKind) -> bool {
        if self.peek_token_is(&token_kind) {
            self.next_token();
            true
        } else {
            self.peek_error(&token_kind);
            false
        }
    }

    fn peek_token_is(&self, token_kind: &TokenKind) -> bool {
        self.peek_token.kind == *token_kind
    }

    fn cur_token_is(&self, token_kind: TokenKind) -> bool {
        self.cur_token.kind == token_kind
    }

    fn errors(&self) -> &Vec<String> {
        &self.errors
    }

    fn peek_error(&mut self, token_kind: &TokenKind) {
        let msg = format!(
            "expected next token to be {:?}, got {:?} instead",
            token_kind, self.peek_token.kind
        );
        self.errors.push(msg);
    }

    fn parse_return_statement(&mut self) -> Option<StatementNode> {
        let stmt = ReturnStatement {
            token: self.cur_token.clone(),
            return_value: Default::default(),
        };

        self.next_token();

        while !self.cur_token_is(TokenKind::Semicolon) {
            self.next_token();
        }

        Some(StatementNode::Return(stmt))
    }

    fn parse_statement(&mut self) -> Option<StatementNode> {
        match self.cur_token.kind {
            TokenKind::Let => self.parse_let_statement(),
            TokenKind::Return => self.parse_return_statement(),
            _ => self.parse_expression_statement(),
        }
    }

    fn parse_let_statement(&mut self) -> Option<StatementNode> {
        let mut stmt = LetStatement {
            token: self.cur_token.clone(),
            name: Default::default(),
            value: Default::default(),
        };

        return if !self.expect_peek(TokenKind::Ident) {
            None
        } else {
            stmt.name = Identifier {
                token: self.cur_token.clone(),
                value: self.cur_token.literal.clone(),
            };

            if !self.expect_peek(TokenKind::Assign) {
                None
            } else {
                self.next_token();
                // TODO: need to implement expression parsing
                while !self.expect_peek(TokenKind::Semicolon) {
                    self.next_token();
                }
                Some(StatementNode::Let(stmt))
            }
        };
    }

    fn parse_expression_statement(&mut self) -> Option<StatementNode> {
        let stmt = ExpressionStatement {
            token: self.cur_token.clone(),
            expression: self.parse_expression(PrecedenceLevel::Lowest),
        };
        if self.peek_token_is(&TokenKind::Semicolon) {
            self.next_token();
        }
        Some(StatementNode::Expression(stmt))
    }

    fn parse_expression(&mut self, precedence_level: PrecedenceLevel) -> Option<ExpressionNode> {
        let prefix = self.prefix_parse_fns.get(&self.cur_token.kind);
        if let Some(prefix_fn) = prefix {
            let mut left_exp = prefix_fn(self);
            while !self.peek_token_is(&TokenKind::Semicolon)
                && (precedence_level as u8) < (self.peek_precedence() as u8)
            {
                let infix_fn = self.infix_parse_fns.get(&self.peek_token.kind);
                if let Some(infix_func) = infix_fn {
                    left_exp = infix_func(
                        self,
                        left_exp.expect("left_exp is None, but it should be Some(ExpressionNode)"),
                    );
                }
            }
            return left_exp;
        };
        self.no_prefix_parse_fn_error(self.cur_token.kind.clone());
        None
    }

    fn no_prefix_parse_fn_error(&mut self, token_kind: TokenKind) {
        let msg = format!("no prefix parse function for '{}' found", token_kind);
        self.errors.push(msg);
    }

    fn parse_identifier(&mut self) -> Option<ExpressionNode> {
        Some(ExpressionNode::IdentifierNode(Identifier {
            token: self.cur_token.clone(),
            value: self.cur_token.literal.clone(),
        }))
    }

    fn parse_integer_literal(&mut self) -> Option<ExpressionNode> {
        let mut literal = IntegerLiteral {
            token: self.cur_token.clone(),
            value: Default::default(),
        };

        return match self.cur_token.literal.parse::<i64>() {
            Ok(value) => {
                literal.value = value;
                Some(ExpressionNode::Integer(literal))
            }
            Err(_) => {
                self.errors.push(format!(
                    "could not parse '{}' as integer",
                    self.cur_token.literal
                ));
                None
            }
        };
    }

    fn parse_prefix_expression(&mut self) -> Option<ExpressionNode> {
        let mut expression = PrefixExpression {
            token: self.cur_token.clone(),
            operator: self.cur_token.literal.clone(),
            right: Default::default(),
        };

        self.next_token();

        match self.parse_expression(PrecedenceLevel::Prefix) {
            Some(right) => {
                expression.right = Box::new(right);
                Some(ExpressionNode::Prefix(expression))
            }
            None => None,
        }
    }

    fn parse_infix_expression(&mut self, left: ExpressionNode) -> Option<ExpressionNode> {
        self.next_token();

        let mut expression = InfixExpression {
            token: self.cur_token.clone(),
            operator: self.cur_token.literal.clone(),
            left: Box::new(left),
            right: Default::default(),
        };

        let precedence = self.cur_precedence();
        self.next_token();
        match self.parse_expression(precedence) {
            Some(right) => {
                expression.right = Box::new(right);
                Some(ExpressionNode::Infix(expression))
            }
            None => None,
        }
    }

    fn register_prefix(&mut self, token_kind: TokenKind, func: PrefixParseFn) {
        self.prefix_parse_fns.insert(token_kind, func);
    }

    fn register_infix(&mut self, token_kind: TokenKind, func: InfixParseFn) {
        self.infix_parse_fns.insert(token_kind, func);
    }

    fn peek_precedence(&self) -> PrecedenceLevel {
        precedence_map(&self.peek_token.kind)
    }

    fn cur_precedence(&self) -> PrecedenceLevel {
        precedence_map(&self.cur_token.kind)
    }
}

#[cfg(test)]
mod tests {
    use std::any;

    use super::Parser;
    use crate::ast::{ExpressionNode, Node, StatementNode};
    use crate::lexer::Lexer;
    use crate::token::TokenKind;

    #[test]
    fn test_let_statements() {
        let input = r#"
        let x = 5;
        let y = 10;
        let foobar = 838383;
        "#;

        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);

        let program = parser.parse_program();

        check_parser_errors(&parser);

        match program {
            Some(program) => {
                assert_eq!(program.statements.len(), 3);
                let tests = vec!["x", "y", "foobar"];
                for (i, expected) in tests.into_iter().enumerate() {
                    let stmt = &program.statements[i];
                    assert_eq!(
                        stmt.token_literal(),
                        "let",
                        "token literal is not let. got={}",
                        stmt.token_literal()
                    );
                    match stmt {
                        StatementNode::Let(let_stmt) => {
                            assert_eq!(
                                let_stmt.name.value, expected,
                                "LetStatement.name.value not '{}'. got={}",
                                expected, let_stmt.name.value
                            );

                            assert_eq!(
                                let_stmt.name.token_literal(),
                                expected,
                                "LetStatement.name.token_literal() not '{}'. got={}",
                                expected,
                                let_stmt.name.token_literal()
                            );
                        }

                        other => {
                            panic!("stmt not LetStatement. got={:?}", other);
                        }
                    }
                }
            }
            None => {
                panic!("parse_program() returned None")
            }
        };
    }

    #[test]
    fn test_return_statement() {
        let input = r#"
        return 5;
        return 10;
        return 993322;
        "#;

        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);

        let program = parser.parse_program();

        check_parser_errors(&parser);

        match program {
            Some(program) => {
                assert_eq!(
                    program.statements.len(),
                    3,
                    "program.statements does not contain 3 statements. got={}",
                    program.statements.len()
                );
                for stmt in program.statements {
                    match stmt {
                        StatementNode::Return(return_stmt) => {
                            assert_eq!(
                                return_stmt.token_literal(),
                                "return",
                                "return_stmt.token_literal() not 'return'. got={}",
                                return_stmt.token_literal()
                            );
                        }

                        other => {
                            panic!("stmt not ReturnStatement. got={:?}", other);
                        }
                    }
                }
            }
            None => {
                panic!("parse_program() returned None")
            }
        };
    }

    #[test]
    fn test_identifier_expression() {
        let input = "foobar;";

        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);

        let program = parser.parse_program();

        check_parser_errors(&parser);

        match program {
            Some(program) => {
                assert_eq!(
                    program.statements.len(),
                    1,
                    "program has not enough statements. got={}",
                    program.statements.len()
                );

                let stmt = &program.statements[0];
                match stmt {
                    StatementNode::Expression(exp_stmt) => {
                        assert!(exp_stmt.expression.is_some(), "exp_stmt.expression is None");

                        match exp_stmt.expression.as_ref().unwrap() {
                            ExpressionNode::IdentifierNode(ident) => {
                                assert_eq!(
                                    ident.value, "foobar",
                                    "ident.value not 'foobar'. got={}",
                                    ident.value
                                );

                                assert_eq!(
                                    ident.token_literal(),
                                    "foobar",
                                    "ident.token_literal() not 'foobar'. got={}",
                                    ident.token_literal()
                                );
                            }
                            other => {
                                panic!("exp not Identifier. got={:?}", other);
                            }
                        }
                    }

                    other => {
                        panic!("stmt not ExpressionStatement. got={:?}", other);
                    }
                }
            }
            None => {
                panic!("parse_program() returned None")
            }
        };
    }

    #[test]
    fn test_integer_literal_expression() {
        let input = "5;";

        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);

        let program = parser.parse_program();

        check_parser_errors(&parser);

        match program {
            Some(program) => {
                assert_eq!(
                    program.statements.len(),
                    1,
                    "program has not enough statements. got={}",
                    program.statements.len()
                );

                let stmt = &program.statements[0];
                match stmt {
                    StatementNode::Expression(exp_stmt) => {
                        assert!(exp_stmt.expression.is_some(), "exp_stmt.expression is None");

                        match exp_stmt.expression.as_ref().unwrap() {
                            ExpressionNode::Integer(integer) => {
                                assert_eq!(
                                    integer.value, 5,
                                    "integer.value not 5. got={}",
                                    integer.value
                                );

                                assert_eq!(
                                    integer.token_literal(),
                                    "5",
                                    "integer.token_literal() not '5'. got={}",
                                    integer.token_literal()
                                );
                            }
                            other => {
                                panic!("exp not IntegerLiteral. got={:?}", other);
                            }
                        }
                    }

                    other => {
                        panic!("stmt not ExpressionStatement. got={:?}", other);
                    }
                }
            }
            None => {
                panic!("parse_program() returned None")
            }
        };
    }

    #[test]
    fn test_parsing_prefix_expressions() {
        let prefix_tests = vec![("!5", "!", 5), ("-15", "-", 15)];

        for test in prefix_tests {
            let lexer = Lexer::new(test.0);
            let mut parser = Parser::new(lexer);

            let program = parser.parse_program().unwrap();

            check_parser_errors(&parser);

            assert_eq!(
                program.statements.len(),
                1,
                "program.statements does not contain 1 statements. got={}",
                program.statements.len()
            );

            match &program.statements[0] {
                StatementNode::Expression(exp_stmt) => {
                    assert!(exp_stmt.expression.is_some(), "exp_stmt.expression is None");

                    match exp_stmt.expression.as_ref().unwrap() {
                        ExpressionNode::Prefix(prefix_exp) => {
                            assert_eq!(
                                prefix_exp.token_literal(),
                                test.1,
                                "prefix_exp
                                .token_literal() is not '{}'. got={}",
                                test.1,
                                prefix_exp.token_literal()
                            );

                            test_integer_literal(&prefix_exp.right, test.2);
                        }
                        other => {
                            panic!(
                                "prefix_exp
                             not Prefix. got={:?}",
                                other
                            );
                        }
                    }
                }

                other => {
                    panic!("stmt not ExpressionStatement. got={:?}", other);
                }
            }
        }
    }

    pub fn check_parser_errors(parser: &Parser) {
        let errors = parser.errors();
        if errors.len() == 0 {
            return;
        }

        eprintln!("parser has {} errors", errors.len());
        for error in errors {
            eprintln!("parser error: {}", error);
        }
        panic!("parser errors found");
    }

    fn test_integer_literal(exp: &ExpressionNode, value: i64) {
        match exp {
            ExpressionNode::Integer(integer) => {
                assert_eq!(
                    integer.value, value,
                    "integer.value not {}. got={}",
                    value, integer.value
                );

                assert_eq!(
                    integer.token_literal(),
                    value.to_string(),
                    "integer.token_literal() not '{}'. got={}",
                    value,
                    integer.token_literal()
                );
            }
            other => {
                panic!("exp not IntegerLiteral. got={:?}", other);
            }
        }
    }

    #[test]
    fn test_parsing_infix_expressions() {
        let infix_tests: Vec<(&str, i64, &str, i64)> = vec![
            ("5 + 5;", 5, "+", 5),
            ("5 - 5;", 5, "-", 5),
            ("5 * 5;", 5, "*", 5),
            ("5 / 5;", 5, "/", 5),
            ("5 > 5;", 5, ">", 5),
            ("5 < 5;", 5, "<", 5),
            ("5 == 5;", 5, "==", 5),
            ("5 != 5;", 5, "!=", 5),
        ];

        for test in infix_tests {
            let lexer = Lexer::new(test.0);
            let mut parser = Parser::new(lexer);

            let program = parser.parse_program().unwrap();

            check_parser_errors(&parser);

            assert_eq!(
                program.statements.len(),
                1,
                "program.statements does not contain 1 statements. got={}",
                program.statements.len()
            );

            match &program.statements[0] {
                StatementNode::Expression(exp_stmt) => {
                    assert!(exp_stmt.expression.is_some(), "exp_stmt.expression is None");

                    let expression = exp_stmt.expression.as_ref().unwrap();

                    test_infix_expression(
                        &expression,
                        Box::new(test.1),
                        test.2.to_string(),
                        Box::new(test.3),
                    );
                }

                other => {
                    panic!("stmt not ExpressionStatement. got={:?}", other);
                }
            }
        }
    }

    #[test]
    fn test_operator_precedence_parsing() {
        let tests = vec![
            ("-a * b", "((-a) * b)"),
            ("!-a", "(!(-a))"),
            ("a + b + c", "((a + b) + c)"),
            ("a + b - c", "((a + b) - c)"),
            ("a * b * c", "((a * b) * c)"),
            ("a * b / c", "((a * b) / c)"),
            ("a + b / c", "(a + (b / c))"),
            ("a + b * c + d / e - f", "(((a + (b * c)) + (d / e)) - f)"),
            ("3 + 4; -5 * 5", "(3 + 4)((-5) * 5)"),
            ("5 > 4 == 3 < 4", "((5 > 4) == (3 < 4))"),
            ("5 < 4 != 3 > 4", "((5 < 4) != (3 > 4))"),
            (
                "3 + 4 * 5 == 3 * 1 + 4 * 5",
                "((3 + (4 * 5)) == ((3 * 1) + (4 * 5)))",
            ),
        ];

        for test in tests {
            let lexer = Lexer::new(test.0);
            let mut parser = Parser::new(lexer);

            let program = parser.parse_program().unwrap();

            check_parser_errors(&parser);

            let actual = program.print_string();
            assert_eq!(actual, test.1, "expected={}, got={}", test.1, actual);
        }
    }

    #[test]
    fn test_boolean_expression() {
        let input = r#"
        true;
        false;
        "#;

        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);

        let program = parser.parse_program().unwrap();

        check_parser_errors(&parser);

        assert_eq!(
            program.statements.len(),
            2,
            "program.statements does not contain 2 statements. got={}",
            program.statements.len()
        );

        let expected_values = vec![(TokenKind::True, "true"), (TokenKind::False, "false")];

        for (index, test) in expected_values.into_iter().enumerate() {
            match &program.statements[index] {
                StatementNode::Expression(exp_stmt) => {
                    assert!(exp_stmt.expression.is_some(), "exp_stmt.expression is None");

                    match exp_stmt.expression.as_ref().unwrap() {
                        ExpressionNode::BooleanNode(boolean) => {
                            assert_eq!(
                                boolean.token.kind,
                                test.0,
                                "boolean.kind not {}. got={}",
                                TokenKind::True,
                                boolean.token.kind
                            );

                            assert_eq!(
                                boolean.token_literal(),
                                test.1,
                                "boolean.token_literal() not '{}'. got={}",
                                test.1,
                                boolean.token_literal()
                            );
                        }
                        other => {
                            panic!("exp not Boolean. got={:?}", other);
                        }
                    }
                }

                other => {
                    panic!("stmt not ExpressionStatement. got={:?}", other);
                }
            }
        }
    }

    fn test_identifier(exp: &ExpressionNode, value: String) {
        match exp {
            ExpressionNode::IdentifierNode(identifier_exp) => {
                assert_eq!(
                    identifier_exp.value, value,
                    "identifier_exp.value not '{}'. got={}",
                    value, identifier_exp.value
                );

                assert_eq!(
                    identifier_exp.token_literal(),
                    value,
                    "identifier_exp.token_literal() not '{}'. got={}",
                    value,
                    identifier_exp.token_literal()
                );
            }
            other => {
                panic!("exp not Identifier. got={:?}", other);
            }
        }
    }

    fn test_literal_expression(exp: &ExpressionNode, expected: Box<dyn any::Any>) {
        match expected.downcast_ref::<String>() {
            Some(exp_string) => test_identifier(exp, exp_string.to_string()),
            None => match expected.downcast_ref::<i64>() {
                Some(int_exp) => {
                    test_integer_literal(exp, *int_exp);
                }
                None => {
                    panic!("type of exp not handled. got={:?}", expected);
                }
            },
        }
    }

    fn test_infix_expression(
        exp: &ExpressionNode,
        left: Box<dyn any::Any>,
        operator: String,
        right: Box<dyn any::Any>,
    ) {
        match exp {
            ExpressionNode::Infix(infix_exp) => {
                test_literal_expression(&infix_exp.left, left);
                assert_eq!(
                    infix_exp.operator, operator,
                    "infix_exp.operator is not '{}'. got={}",
                    operator, infix_exp.operator
                );
                test_literal_expression(&infix_exp.right, right);
            }
            other => {
                panic!("exp not Infix. got={:?}", other);
            }
        }
    }
}
