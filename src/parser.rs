use crate::ast::{Identifier, LetStatement, Program, StatementNode};
use crate::lexer::Lexer;
use crate::token::{Token, TokenKind};

pub struct Parser {
    lexer: Lexer,
    pub cur_token: Token,
    pub peek_token: Token,
}

impl Parser {
    fn new(lexer: Lexer) -> Parser {
        let mut parser = Parser {
            lexer,
            cur_token: Default::default(),
            peek_token: Default::default(),
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

        while self.cur_token.kind != TokenKind::EOF {
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

                while !self.expect_peek(TokenKind::Semicolon) {
                    self.next_token();
                }
                Some(StatementNode::Let(stmt))
            }
        };
    }

    fn expect_peek(&mut self, token_kind: TokenKind) -> bool {
        if self.peek_token_is(token_kind) {
            self.next_token();
            return true;
        } else {
            return false;
        }
    }

    fn peek_token_is(&self, token_kind: TokenKind) -> bool {
        self.peek_token.kind == token_kind
    }

    fn cur_token_is(&self, token_kind: TokenKind) -> bool {
        self.cur_token.kind == token_kind
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
                    }
                }
            }
            None => {
                panic!("parse_program() returned None")
            }
        };
    }
}
