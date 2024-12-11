use std::collections::HashMap;

use crate::ast::{
    ExpressionNode, ExpressionStatement, Identifier, IntegerLiteral, LetStatement, Program,
    ReturnStatement, StatementNode,
};
use crate::lexer::Lexer;
use crate::token::{Token, TokenKind};

type PrefixParseFn = fn(&mut Parser) -> Option<ExpressionNode>;
type InfixParseFn = fn(&mut Parser, ExpressionNode) -> Option<ExpressionNode>;

enum PrecedenceLevel {
    Lowest = 0,
    Equals = 1,      // ==
    LessGreater = 2, // > or <
    Sum = 3,         // +
    Product = 4,
    Prefix = 5,
    Call = 6,
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

        parser.next_token();
        parser.next_token();

        return parser;
    }

    fn next_token(&mut self) {
        self.cur_token = self.peek_token.clone();
        self.peek_token = self.lexer.next_token();
    }

    fn parse_program(&mut self) -> Option<Program> {
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

    fn register_prefix(&mut self, token_kind: TokenKind, func: PrefixParseFn) {
        self.prefix_parse_fns.insert(token_kind, func);
    }

    fn register_infix(&mut self, token_kind: TokenKind, func: InfixParseFn) {
        self.infix_parse_fns.insert(token_kind, func);
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

    fn parse_expression(&mut self, precedence: PrecedenceLevel) -> Option<ExpressionNode> {
        let prefix = self.prefix_parse_fns.get(&self.cur_token.kind);
        if let Some(prefix_fn) = prefix {
            let mut left_exp = prefix_fn(self);
            // while !self.peek_token_is(&TokenKind::Semicolon) && precedence < self.peek_precedence() {
            //     let infix = self.infix_parse_fns.get(&self.peek_token.kind);
            //     if let Some(infix_fn) = infix {
            //         self.next_token();
            //         left_exp = infix_fn(self, left_exp.unwrap());
            //     }
            // }
            return left_exp;
        };
        None
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
}

#[cfg(test)]
mod tests {
    use super::Parser;
    use crate::ast::{ExpressionNode, Node, StatementNode};
    use crate::lexer::Lexer;

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

    fn check_parser_errors(parser: &Parser) {
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
}
