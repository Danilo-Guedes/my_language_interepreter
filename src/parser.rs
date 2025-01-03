use std::collections::HashMap;

use crate::ast::{
    BlockStatement, Boolean, ExpressionNode, ExpressionStatement, FunctionLiteral, Identifier,
    IfExpression, InfixExpression, IntegerLiteral, LetStatement, PrefixExpression, Program,
    ReturnStatement, StatementNode,
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

        //PREFIX
        parser.register_prefix(TokenKind::Ident, Self::parse_identifier);
        parser.register_prefix(TokenKind::Int, Self::parse_integer_literal);
        parser.register_prefix(TokenKind::Bang, Self::parse_prefix_expression);
        parser.register_prefix(TokenKind::Minus, Self::parse_prefix_expression);
        parser.register_prefix(TokenKind::True, Self::parse_boolean);
        parser.register_prefix(TokenKind::False, Self::parse_boolean);
        parser.register_prefix(TokenKind::LParen, Self::parse_grouped_expression);
        parser.register_prefix(TokenKind::If, Self::parse_if_expression);
        parser.register_prefix(TokenKind::Function, Self::parse_function_literal);

        //INFIX
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

    fn parse_boolean(&mut self) -> Option<ExpressionNode> {
        Some(ExpressionNode::BooleanNode(Boolean {
            token: self.cur_token.clone(),
            value: self.cur_token_is(TokenKind::True),
        }))
    }

    fn parse_grouped_expression(&mut self) -> Option<ExpressionNode> {
        self.next_token();

        let exp = self.parse_expression(PrecedenceLevel::Lowest);

        if !self.expect_peek(TokenKind::RParen) {
            return None;
        }

        exp
    }

    fn parse_if_expression(&mut self) -> Option<ExpressionNode> {
        let mut expression = IfExpression {
            token: self.cur_token.clone(),
            alternative: None,
            condition: Default::default(),
            consequence: Default::default(),
        };

        if !self.expect_peek(TokenKind::LParen) {
            return None;
        }

        self.next_token();

        expression.condition = Box::new(
            self.parse_expression(PrecedenceLevel::Lowest)
                .expect("error parsing condition"),
        );

        if !self.expect_peek(TokenKind::RParen) {
            return None;
        }

        if !self.expect_peek(TokenKind::LBrace) {
            return None;
        }

        expression.consequence = self.parse_block_statement();

        if self.peek_token_is(&TokenKind::Else) {
            self.next_token();

            if !self.expect_peek(TokenKind::LBrace) {
                return None;
            }

            expression.alternative = Some(self.parse_block_statement());
        }

        Some(ExpressionNode::IfExpressionNode(expression))
    }

    fn parse_block_statement(&mut self) -> BlockStatement {
        let mut block = BlockStatement {
            token: self.cur_token.clone(),
            statements: Vec::new(),
        };

        self.next_token();

        while !self.cur_token_is(TokenKind::RBrace) && !self.cur_token_is(TokenKind::EOF) {
            if let Some(stmt) = self.parse_statement() {
                block.statements.push(stmt);
            }
            self.next_token();
        }

        block
    }
    fn parse_function_literal(&mut self) -> Option<ExpressionNode> {
        let mut func_lit = FunctionLiteral {
            token: self.cur_token.clone(),
            parameters: Vec::new(),
            body: Default::default(),
        };

        if !self.expect_peek(TokenKind::LParen) {
            return None;
        }

        func_lit.parameters = self
            .parse_function_parameters()
            .expect("error parsing parameters");

        if !self.expect_peek(TokenKind::LBrace) {
            return None;
        }

        func_lit.body = self.parse_block_statement();

        Some(ExpressionNode::Function(func_lit))
    }

    fn parse_function_parameters(&mut self) -> Option<Vec<Identifier>> {
        let mut identifiers = Vec::new();

        if self.peek_token_is(&TokenKind::RParen) {
            self.next_token();
            return Some(identifiers);
        }

        self.next_token();

        let ident = Identifier {
            token: self.cur_token.clone(),
            value: self.cur_token.literal.clone(),
        };

        identifiers.push(ident);

        while self.peek_token_is(&TokenKind::Comma) {
            self.next_token();
            self.next_token();
            let ident = Identifier {
                token: self.cur_token.clone(),
                value: self.cur_token.literal.clone(),
            };
            identifiers.push(ident);
        }

        if !self.expect_peek(TokenKind::RParen) {
            return None;
        }

        Some(identifiers)
    }
}

#[cfg(test)]
mod tests {
    use std::any;

    use super::Parser;
    use crate::ast::{ExpressionNode, Identifier, Node, StatementNode};
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
        let prefix_tests: Vec<(&str, &str, Box<dyn any::Any>)> = vec![
            ("!5", "!", Box::new(5)),
            ("-15", "-", Box::new(15)),
            ("!true", "!", Box::new(true)),
            ("!false", "!", Box::new(false)),
        ];

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

                            test_literal_expression(&prefix_exp.right, test.2);
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

    #[test]
    fn test_parsing_infix_expressions() {
        let infix_tests: Vec<(&str, Box<dyn any::Any>, &str, Box<dyn any::Any>)> = vec![
            ("5 + 5;", Box::new(5), "+", Box::new(5)),
            ("5 - 5;", Box::new(5), "-", Box::new(5)),
            ("5 * 5;", Box::new(5), "*", Box::new(5)),
            ("5 / 5;", Box::new(5), "/", Box::new(5)),
            ("5 > 5;", Box::new(5), ">", Box::new(5)),
            ("5 < 5;", Box::new(5), "<", Box::new(5)),
            ("5 == 5;", Box::new(5), "==", Box::new(5)),
            ("5 != 5;", Box::new(5), "!=", Box::new(5)),
            ("true == true", Box::new(true), "==", Box::new(true)),
            ("true != false", Box::new(true), "!=", Box::new(false)),
            ("false == false", Box::new(false), "==", Box::new(false)),
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
            ("true", "true"),
            ("false", "false"),
            ("3 > 5 == false", "((3 > 5) == false)"),
            ("3 < 5 == true", "((3 < 5) == true)"),
            ("1 + (2 + 3) + 4", "((1 + (2 + 3)) + 4)"),
            ("(5 + 5) * 2", "((5 + 5) * 2)"),
            ("2 / (5 + 5)", "(2 / (5 + 5))"),
            ("-(5 + 5)", "(-(5 + 5))"),
            ("!(true == true)", "(!(true == true))"),
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

    #[test]
    fn test_if_expression() {
        let input = r#"
        if (x < y) { x }
        "#;

        let lexer = Lexer::new(input);
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
                    ExpressionNode::IfExpressionNode(if_exp) => {
                        test_infix_expression(
                            &if_exp.condition,
                            Box::new("x"),
                            String::from("<"),
                            Box::new("y"),
                        );

                        assert_eq!(
                            if_exp.consequence.statements.len(),
                            1,
                            "consequence is not 1 statements. got={}",
                            if_exp.consequence.statements.len()
                        );

                        match &if_exp.consequence.statements[0] {
                            StatementNode::Expression(consequence) => {
                                test_identifier(
                                    consequence
                                        .expression
                                        .as_ref()
                                        .expect("Error parsing consequence"),
                                    "x".to_string(),
                                );
                            }
                            other => {
                                panic!("stmt not ExpressionStatement. got={:?}", other);
                            }
                        }

                        assert!(if_exp.alternative.is_none(), "alternative is not None");
                    }
                    other => {
                        panic!("exp not IfExpression. got={:?}", other);
                    }
                }
            }

            other => {
                panic!("stmt not ExpressionStatement. got={:?}", other);
            }
        }
    }
    #[test]
    fn test_if_else_expression() {
        let input = r#"
        if (x < y) { x } else { y }
        "#;

        let lexer = Lexer::new(input);
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
                    ExpressionNode::IfExpressionNode(if_exp) => {
                        test_infix_expression(
                            &if_exp.condition,
                            Box::new("x"),
                            String::from("<"),
                            Box::new("y"),
                        );

                        assert_eq!(
                            if_exp.consequence.statements.len(),
                            1,
                            "consequence is not 1 statements. got={}",
                            if_exp.consequence.statements.len()
                        );

                        assert_eq!(
                            if_exp.alternative.as_ref().unwrap().statements.len(),
                            1,
                            "alternative is not 1 statements. got={}",
                            if_exp.alternative.as_ref().unwrap().statements.len()
                        );

                        match &if_exp.consequence.statements[0] {
                            StatementNode::Expression(consequence) => {
                                test_identifier(
                                    consequence
                                        .expression
                                        .as_ref()
                                        .expect("Error parsing consequence"),
                                    "x".to_string(),
                                );
                            }
                            other => {
                                panic!("stmt not ExpressionStatement. got={:?}", other);
                            }
                        }

                        match &if_exp.alternative.as_ref().unwrap().statements[0] {
                            StatementNode::Expression(alternative) => {
                                test_identifier(
                                    alternative
                                        .expression
                                        .as_ref()
                                        .expect("Error parsing alternative"),
                                    "y".to_string(),
                                );
                            }
                            other => {
                                panic!("stmt not ExpressionStatement. got={:?}", other);
                            }
                        }
                    }
                    other => {
                        panic!("exp not IfExpression. got={:?}", other);
                    }
                }
            }

            other => {
                panic!("stmt not ExpressionStatement. got={:?}", other);
            }
        }
    }

    #[test]
    fn test_function_literal_parsing() {
        let input = "fn(x, y) { x + y; }";

        let lexer = Lexer::new(input);
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
                    ExpressionNode::Function(function) => {
                        assert_eq!(
                            function.parameters.len(),
                            2,
                            "function literal parameters wrong. want 2. got={}",
                            function.parameters.len()
                        );

                        match &function.parameters[0] {
                            Identifier { token, value } => {
                                assert_eq!(
                                    value, "x",
                                    "function literal parameter is not 'x'. got={}",
                                    value
                                );
                                assert_eq!(
                                    token.literal, "x",
                                    "function literal parameter is not 'x'. got={}",
                                    token.literal
                                )
                            }
                        }

                        match &function.parameters[1] {
                            Identifier { token, value } => {
                                assert_eq!(
                                    value, "y",
                                    "function literal parameter is not 'y'. got={}",
                                    value
                                );
                                assert_eq!(
                                    token.literal, "y",
                                    "function literal parameter is not 'y'. got={}",
                                    token.literal
                                )
                            }
                        }

                        assert_eq!(
                            function.body.statements.len(),
                            1,
                            "function.body.statements has not 1 statements. got={}",
                            function.body.statements.len()
                        );

                        match &function.body.statements[0] {
                            StatementNode::Expression(body_exp) => {
                                test_infix_expression(
                                    &body_exp.expression.as_ref().unwrap(),
                                    Box::new("x"),
                                    "+".to_string(),
                                    Box::new("y"),
                                );
                            }
                            other => {
                                panic!(
                                    "function body stmt is not ExpressionStatement. got={:?}",
                                    other
                                );
                            }
                        }
                    }
                    other => {
                        panic!("exp not FunctionLiteral. got={:?}", other);
                    }
                }
            }

            other => {
                panic!("stmt not ExpressionStatement. got={:?}", other);
            }
        }
    }

    #[test]
    fn test_function_paramenter_parsing() {
        let tests = vec![
            ("fn() {};", vec![]),
            ("fn(x) {};", vec!["x"]),
            ("fn(x, y, z) {};", vec!["x", "y", "z"]),
        ];

        for test in tests {
            let lexer = Lexer::new(test.0);
            let mut parser = Parser::new(lexer);

            let program = parser.parse_program().unwrap();

            check_parser_errors(&parser);

            match &program.statements[0] {
                StatementNode::Expression(exp_stmt) => {
                    assert!(exp_stmt.expression.is_some(), "exp_stmt.expression is None");

                    match exp_stmt.expression.as_ref().unwrap() {
                        ExpressionNode::Function(function) => {
                            assert_eq!(
                                function.parameters.len(),
                                test.1.len(),
                                "length parameters wrong. want {}, got={}",
                                test.1.len(),
                                function.parameters.len()
                            );

                            for (i, param) in test.1.into_iter().enumerate() {
                                match &function.parameters[i] {
                                    Identifier { token, value } => {
                                        assert_eq!(
                                            value, param,
                                            "function literal parameter is not '{}'. got={}",
                                            param, value
                                        );
                                        assert_eq!(
                                            token.literal, param,
                                            "function literal parameter is not '{}'. got={}",
                                            param, token.literal
                                        )
                                    }
                                }
                            }
                        }
                        other => {
                            panic!("exp not FunctionLiteral. got={:?}", other);
                        }
                    }
                }

                other => {
                    panic!("stmt not ExpressionStatement. got={:?}", other);
                }
            }
        }
    }

    #[test]
    fn test_call_expression_parsing() {
        let input = "add(1, 2 * 3, 4 + 5);";

        let lexer = Lexer::new(input);
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
                    ExpressionNode::Call(call_exp) => {
                        test_identifier(&call_exp.function, "add".to_string());

                        assert_eq!(
                            call_exp.arguments.len(),
                            3,
                            "wrong length of arguments. got={}",
                            call_exp.arguments.len()
                        );

                        test_literal_expression(&call_exp.arguments[0], Box::new(1));
                        test_infix_expression(
                            &call_exp.arguments[1],
                            Box::new(2),
                            "*".to_string(),
                            Box::new(3),
                        );
                        test_infix_expression(
                            &call_exp.arguments[2],
                            Box::new(4),
                            "+".to_string(),
                            Box::new(5),
                        );
                    }
                    other => {
                        panic!("exp not CallExpression. got={:?}", other);
                    }
                }
            }

            other => {
                panic!("stmt not ExpressionStatement. got={:?}", other);
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
                None => match expected.downcast_ref::<bool>() {
                    Some(bool) => test_boolean_literal(exp, bool.to_owned()),
                    None => (),
                },
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

    fn test_boolean_literal(exp: &ExpressionNode, value: bool) {
        match exp {
            ExpressionNode::BooleanNode(bool_exp) => {
                assert_eq!(
                    bool_exp.value, value,
                    "boolean.value not {}. got={}",
                    value, bool_exp.value
                );

                assert_eq!(
                    bool_exp.token_literal(),
                    value.to_string(),
                    "boolean.token_literal() not '{}'. got={}",
                    value,
                    bool_exp.token_literal()
                );
            }
            other => {
                panic!("exp not Boolean. got={:?}", other);
            }
        }
    }
}
