use crate::token::{Token, TokenKind};

struct Lexer {
    input: Vec<char>,
    position: usize,
    read_position: usize,
    ch: char,
}

impl Lexer {
    pub fn new(input: &str) -> Lexer {
        let vec_of_chars = input.chars().collect();
        let mut lexer = Lexer {
            input: vec_of_chars,
            position: 0,
            read_position: 0,
            ch: Default::default(), // this initializes the char to '\0' which is a null character
        };

        lexer.read_char();

        lexer
    }
    fn read_char(&mut self) {
        if self.read_position >= self.input.len() {
            self.ch = '\0';
        } else {
            self.ch = self.input[self.read_position];
        }
        self.position = self.read_position;
        self.read_position += 1;
    }
    pub fn next_token(&mut self) -> Token {
        let token = match self.ch {
            '=' => Lexer::new_token(TokenKind::Assign, self.ch),
            ';' => Lexer::new_token(TokenKind::Semicolon, self.ch),
            '(' => Lexer::new_token(TokenKind::LParen, self.ch),
            ')' => Lexer::new_token(TokenKind::RParen, self.ch),
            ',' => Lexer::new_token(TokenKind::Comma, self.ch),
            '+' => Lexer::new_token(TokenKind::Plus, self.ch),
            '{' => Lexer::new_token(TokenKind::LBrace, self.ch),
            '}' => Lexer::new_token(TokenKind::RBrace, self.ch),
            '\0' => Token {
                kind: TokenKind::EOF,
                literal: "".to_string(),
            },
            _ => Lexer::new_token(TokenKind::Illegal, self.ch),
        };

        self.read_char();

        token
    }
    fn new_token(kind: TokenKind, ch: char) -> Token {
        Token {
            kind,
            literal: ch.to_string(),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::token::{Token, TokenKind};

    use super::Lexer;

    #[test]
    fn test_next_token() {
        let input: &str = "=+(){},;";

        let expected: Vec<Token> = vec![
            Token {
                kind: TokenKind::Assign,
                literal: "=".to_string(),
            },
            Token {
                kind: TokenKind::Plus,
                literal: "+".to_string(),
            },
            Token {
                kind: TokenKind::LParen,
                literal: "(".to_string(),
            },
            Token {
                kind: TokenKind::RParen,
                literal: ")".to_string(),
            },
            Token {
                kind: TokenKind::LBrace,
                literal: "{".to_string(),
            },
            Token {
                kind: TokenKind::RBrace,
                literal: "}".to_string(),
            },
            Token {
                kind: TokenKind::Comma,
                literal: ",".to_string(),
            },
        ];

        let mut lexer = Lexer::new(input);

        for (idx, token) in expected.into_iter().enumerate() {
            let received_token = lexer.next_token();
            assert_eq!(
                token.kind, received_token.kind,
                "tests[{}] - token type wrong. expected={}, got={}",
                idx, token.kind, received_token.kind
            );

            assert_eq!(
                token.literal, received_token.literal,
                "tests[{}] - literal wrong. expected={}, got={}",
                idx, token.literal, received_token.literal
            );
        }
    }
}
