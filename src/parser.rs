use crate::ast::Program;
use crate::lexer::Lexer;
use crate::token::{Token, TokenKind};


pub struct Parser {
    lexer: Lexer,
    cur_token: Token,
    peek_token: Token,
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
            let stmt = self.parse_statement();
            if stmt != None {
                program.statements.push(stmt);
            }
            self.next_token();
        }

        return program;
    }
}