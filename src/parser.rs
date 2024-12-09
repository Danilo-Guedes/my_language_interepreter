use crate::ast::{Identifier, LetStatement, Program, ReturnStatement, StatementNode};
use crate::lexer::Lexer;
use crate::token::{Token, TokenKind};

pub struct Parser {
    lexer: Lexer,
    pub cur_token: Token,
    pub peek_token: Token,
    errors: Vec<String>,
}

impl Parser {
    fn new(lexer: Lexer) -> Parser {
        let mut parser = Parser {
            lexer,
            cur_token: Default::default(),
            peek_token: Default::default(),
            errors: Vec::new(),
        };

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
            _ => None,
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
            return true;
        } else {
            self.peek_error(&token_kind);
            return false;
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

        return Some(StatementNode::Return(stmt));
    }
}

#[cfg(test)]
mod tests {
    use super::Parser;
    use crate::ast::{Node, StatementNode};
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
}
