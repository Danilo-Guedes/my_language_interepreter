use crate::token::{lookup_keywords, Token, TokenKind};

pub struct Lexer {
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
        self.skip_whitespace_and_comments();

        let token = match self.ch {
            '=' => {
                if self.peek_char() == '=' {
                    self.read_char();
                    Token {
                        kind: TokenKind::EQ,
                        literal: "==".to_string(),
                    }
                } else {
                    Lexer::new_token(TokenKind::Assign, self.ch)
                }
            }
            ';' => Lexer::new_token(TokenKind::Semicolon, self.ch),
            ':' => Lexer::new_token(TokenKind::Colon, self.ch),
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
            '!' => {
                if self.peek_char() == '=' {
                    self.read_char();
                    Token {
                        kind: TokenKind::NotEQ,
                        literal: "!=".to_string(),
                    }
                } else {
                    Lexer::new_token(TokenKind::Bang, self.ch)
                }
            }
            '*' => Lexer::new_token(TokenKind::Asterisk, self.ch),
            '/' => Lexer::new_token(TokenKind::Slash, self.ch),
            '<' => Lexer::new_token(TokenKind::LT, self.ch),
            '>' => Lexer::new_token(TokenKind::GT, self.ch),
            '"' => Token {
                kind: TokenKind::String,
                literal: self.read_string(),
            },
            '[' => Lexer::new_token(TokenKind::LBracket, self.ch),
            ']' => Lexer::new_token(TokenKind::RBracket, self.ch),
            _ => {
                return if Lexer::is_letter(self.ch) {
                    let literal = self.read_identifier();
                    let kind = lookup_keywords(&literal);
                    Token { kind, literal }
                } else if Lexer::is_digit(self.ch) {
                    let literal = self.read_number();
                    return Token {
                        kind: TokenKind::Int,
                        literal,
                    };
                } else {
                    return Lexer::new_token(TokenKind::Illegal, self.ch);
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

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            while self.ch.is_ascii_whitespace() {
                self.read_char();
            }
            // line comment: `//` to end of line
            if self.ch == '/' && self.peek_char() == '/' {
                self.skip_comment();
            } else {
                break;
            }
        }
    }

    fn skip_comment(&mut self) {
        while self.ch != '\n' && self.ch != '\0' {
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

    fn read_string(&mut self) -> String {
        let position = self.position + 1;
        self.read_char();

        while self.ch != '"' && self.ch != '\0' {
            self.read_char();
        }

        let string_slice = &self.input[position..self.position];
        string_slice.iter().collect()
    }

    fn peek_char(&self) -> char {
        if self.read_position >= self.input.len() {
            '\0'
        } else {
            self.input[self.read_position]
        }
    }
}

#[cfg(test)]
mod test {
    use crate::token::{Token, TokenKind};

    use super::Lexer;

    #[test]
    fn test_next_token() {
        let input: &str = r#"
            let five = 5;
            let ten = 10;

            let add = fn(x, y) {
                x + y;
            };

            let result = add(five, ten);
            !-/*5;
            5 < 10 > 5;

            if (5 < 10) {
                return true;
            } else {
                return false;
            }

            10 == 10;
            10 != 9;
            "foobar"
            "foo bar"
            [1, 2];
            {"foo": "bar"};
            let five = 5;// a comment
            let ten = 10; // another comment
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
                literal: "10".to_string(),
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
                kind: TokenKind::Int,
                literal: "10".to_string(),
            },
            Token {
                kind: TokenKind::EQ,
                literal: "==".to_string(),
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
                kind: TokenKind::Int,
                literal: "10".to_string(),
            },
            Token {
                kind: TokenKind::NotEQ,
                literal: "!=".to_string(),
            },
            Token {
                kind: TokenKind::Int,
                literal: "9".to_string(),
            },
            Token {
                kind: TokenKind::Semicolon,
                literal: ";".to_string(),
            },
            Token {
                kind: TokenKind::String,
                literal: "foobar".to_string(),
            },
            Token {
                kind: TokenKind::String,
                literal: "foo bar".to_string(),
            },
            Token {
                kind: TokenKind::LBracket,
                literal: "[".to_string(),
            },
            Token {
                kind: TokenKind::Int,
                literal: "1".to_string(),
            },
            Token {
                kind: TokenKind::Comma,
                literal: ",".to_string(),
            },
            Token {
                kind: TokenKind::Int,
                literal: "2".to_string(),
            },
            Token {
                kind: TokenKind::RBracket,
                literal: "]".to_string(),
            },
            Token {
                kind: TokenKind::Semicolon,
                literal: ";".to_string(),
            },
            Token {
                kind: TokenKind::LBrace,
                literal: "{".to_string(),
            },
            Token {
                kind: TokenKind::String,
                literal: "foo".to_string(),
            },
            Token {
                kind: TokenKind::Colon,
                literal: ":".to_string(),
            },
            Token {
                kind: TokenKind::String,
                literal: "bar".to_string(),
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
