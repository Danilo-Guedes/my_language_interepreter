use crate::token::{lookup_keywords, Token, TokenKind};

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
        self.skip_whitespace();

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
            '-' => Lexer::new_token(TokenKind::Minus, self.ch),
            '!' => Lexer::new_token(TokenKind::Bang, self.ch),
            '*' => Lexer::new_token(TokenKind::Asterisk, self.ch),
            '/' => Lexer::new_token(TokenKind::Slash, self.ch),
            '<' => Lexer::new_token(TokenKind::LT, self.ch),
            '>' => Lexer::new_token(TokenKind::GT, self.ch),

            _ => {
                return if Lexer::is_letter(self.ch) {
                    let literal = self.read_identifier();
                    let kind = lookup_keywords(&literal);
                    Token { kind, literal }
                } else if Lexer::is_digit(self.ch) {
                    let literal = self.read_number();
                    Token {
                        kind: TokenKind::Int,
                        literal,
                    }
                } else {
                    Lexer::new_token(TokenKind::Illegal, self.ch)
                }
            }
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
    fn is_letter(ch: char) -> bool {
        ch.is_ascii_alphabetic() || ch == '_'
    }
    fn read_identifier(&mut self) -> String {
        let mut identifier = String::new();
        while Lexer::is_letter(self.ch) {
            identifier.push(self.ch);
            self.read_char();
        }
        identifier
    }

    fn skip_whitespace(&mut self) {
        while self.ch.is_ascii_whitespace() {
            self.read_char();
        }
    }

    fn is_digit(ch: char) -> bool {
        ch.is_ascii_digit()
    }

    fn read_number(&mut self) -> String {
        let mut number = String::new();
        while Lexer::is_digit(self.ch) {
            number.push(self.ch);
            self.read_char();
        }
        number
    }
}

#[cfg(test)]
mod test {
    use crate::token::{Token, TokenKind};

    use super::Lexer;

    #[test]
    fn test_next_token_simple() {
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

    #[test]
    fn test_next_token_with_language() {
        let input: &str = r#"
                let five = 5;
                let ten = 10;

                let add = fn(x, y) {
                    x + y;
                };

                let result = add(five, ten);
            "#;

        let expected: Vec<Token> = vec![
            Token {
                kind: TokenKind::Let,
                literal: "let".to_string(),
            },
            Token {
                kind: TokenKind::Ident,
                literal: "five".to_string(),
            },
            Token {
                kind: TokenKind::Assign,
                literal: "=".to_string(),
            },
            Token {
                kind: TokenKind::Int,
                literal: "5".to_string(),
            },
            Token {
                kind: TokenKind::Semicolon,
                literal: ";".to_string(),
            },
            Token {
                kind: TokenKind::Let,
                literal: "let".to_string(),
            },
            Token {
                kind: TokenKind::Ident,
                literal: "ten".to_string(),
            },
            Token {
                kind: TokenKind::Assign,
                literal: "=".to_string(),
            },
            Token {
                kind: TokenKind::Int,
                literal: "10".to_string(),
            },
            Token {
                kind: TokenKind::Semicolon,
                literal: ";".to_string(),
            },
            Token {
                kind: TokenKind::Let,
                literal: "let".to_string(),
            },
            Token {
                kind: TokenKind::Ident,
                literal: "add".to_string(),
            },
            Token {
                kind: TokenKind::Assign,
                literal: "=".to_string(),
            },
            Token {
                kind: TokenKind::Function,
                literal: "fn".to_string(),
            },
            Token {
                kind: TokenKind::LParen,
                literal: "(".to_string(),
            },
            Token {
                kind: TokenKind::Ident,
                literal: "x".to_string(),
            },
            Token {
                kind: TokenKind::Comma,
                literal: ",".to_string(),
            },
            Token {
                kind: TokenKind::Ident,
                literal: "y".to_string(),
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
                kind: TokenKind::Ident,
                literal: "x".to_string(),
            },
            Token {
                kind: TokenKind::Plus,
                literal: "+".to_string(),
            },
            Token {
                kind: TokenKind::Ident,
                literal: "y".to_string(),
            },
            Token {
                kind: TokenKind::Semicolon,
                literal: ";".to_string(),
            },
            Token {
                kind: TokenKind::RBrace,
                literal: "}".to_string(),
            },
            Token {
                kind: TokenKind::Semicolon,
                literal: ";".to_string(),
            },
            Token {
                kind: TokenKind::Let,
                literal: "let".to_string(),
            },
            Token {
                kind: TokenKind::Ident,
                literal: "result".to_string(),
            },
            Token {
                kind: TokenKind::Assign,
                literal: "=".to_string(),
            },
            Token {
                kind: TokenKind::Ident,
                literal: "add".to_string(),
            },
            Token {
                kind: TokenKind::LParen,
                literal: "(".to_string(),
            },
            Token {
                kind: TokenKind::Ident,
                literal: "five".to_string(),
            },
            Token {
                kind: TokenKind::Comma,
                literal: ",".to_string(),
            },
            Token {
                kind: TokenKind::Ident,
                literal: "ten".to_string(),
            },
            Token {
                kind: TokenKind::RParen,
                literal: ")".to_string(),
            },
            Token {
                kind: TokenKind::Semicolon,
                literal: ";".to_string(),
            },
            Token {
                kind: TokenKind::EOF,
                literal: "".to_string(),
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

    #[test]
    fn test_next_token_with_special_characters() {
        let input: &str = r#"
               !-/*5;
               5 < 10 > 5;
            "#;

        let expected: Vec<Token> = vec![
            Token {
                kind: TokenKind::Bang,
                literal: "!".to_string(),
            },
            Token {
                kind: TokenKind::Minus,
                literal: "-".to_string(),
            },
            Token {
                kind: TokenKind::Slash,
                literal: "/".to_string(),
            },
            Token {
                kind: TokenKind::Asterisk,
                literal: "*".to_string(),
            },
            Token {
                kind: TokenKind::Int,
                literal: "5".to_string(),
            },
            Token {
                kind: TokenKind::Semicolon,
                literal: ";".to_string(),
            },
            Token {
                kind: TokenKind::Int,
                literal: "5".to_string(),
            },
            Token {
                kind: TokenKind::LT,
                literal: "<".to_string(),
            },
            Token {
                kind: TokenKind::Int,
                literal: "10".to_string(),
            },
            Token {
                kind: TokenKind::GT,
                literal: ">".to_string(),
            },
            Token {
                kind: TokenKind::Int,
                literal: "5".to_string(),
            },
            Token {
                kind: TokenKind::Semicolon,
                literal: ";".to_string(),
            },
            Token {
                kind: TokenKind::EOF,
                literal: "".to_string(),
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

    #[test]
    fn test_next_token_with_keywords() {
        let input: &str = r#"
              if (5 < 0) {
              return true;
              } else {
               return false;
               }
            "#;

        let expected: Vec<Token> = vec![
            Token {
                kind: TokenKind::If,
                literal: "if".to_string(),
            },
            Token {
                kind: TokenKind::LParen,
                literal: "(".to_string(),
            },
            Token {
                kind: TokenKind::Int,
                literal: "5".to_string(),
            },
            Token {
                kind: TokenKind::LT,
                literal: "<".to_string(),
            },
            Token {
                kind: TokenKind::Int,
                literal: "0".to_string(),
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
                kind: TokenKind::Return,
                literal: "return".to_string(),
            },
            Token {
                kind: TokenKind::True,
                literal: "true".to_string(),
            },
            Token {
                kind: TokenKind::Semicolon,
                literal: ";".to_string(),
            },
            Token {
                kind: TokenKind::RBrace,
                literal: "}".to_string(),
            },
            Token {
                kind: TokenKind::Else,
                literal: "else".to_string(),
            },
            Token {
                kind: TokenKind::LBrace,
                literal: "{".to_string(),
            },
            Token {
                kind: TokenKind::Return,
                literal: "return".to_string(),
            },
            Token {
                kind: TokenKind::False,
                literal: "false".to_string(),
            },
            Token {
                kind: TokenKind::Semicolon,
                literal: ";".to_string(),
            },
            Token {
                kind: TokenKind::RBrace,
                literal: "}".to_string(),
            },
            Token {
                kind: TokenKind::EOF,
                literal: "".to_string(),
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
