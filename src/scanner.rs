use crate::token::{Token, TokenType};

#[derive(Debug)]
pub struct Scanner {
    source: String,
    start: usize,
    current: usize,
    line: usize,
}

impl Scanner {
    pub fn new(source: &str) -> Self {
        Self {
            source: source.to_string(),
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan_token(&mut self) -> Token {
        self.skip_whitespace();
        self.start = self.current;

        if self.is_at_end() {
            return self.make_token(TokenType::Eof);
        }

        let ch = self.advance();
        match ch {
            b'(' => self.make_token(TokenType::LeftParen),
            b')' => self.make_token(TokenType::RightParen),
            b'{' => self.make_token(TokenType::LeftBrace),
            b'}' => self.make_token(TokenType::RightBrace),
            b',' => self.make_token(TokenType::Comma),
            b'.' => self.make_token(TokenType::Dot),
            b'-' => self.make_token(TokenType::Minus),
            b'+' => self.make_token(TokenType::Plus),
            b';' => self.make_token(TokenType::Semicolon),
            b'*' => self.make_token(TokenType::Star),
            b'=' => {
                if self.matches(b'=') {
                    self.make_token(TokenType::Equals)
                } else {
                    self.make_token(TokenType::Assign)
                }
            }
            b'!' => {
                if self.matches(b'=') {
                    self.make_token(TokenType::BangEqual)
                } else {
                    self.make_token(TokenType::Bang)
                }
            }
            b'<' => {
                if self.matches(b'=') {
                    self.make_token(TokenType::LessEqual)
                } else {
                    self.make_token(TokenType::Less)
                }
            }
            b'>' => {
                if self.matches(b'=') {
                    self.make_token(TokenType::GreaterEqual)
                } else {
                    self.make_token(TokenType::Greater)
                }
            }
            b'/' => self.make_token(TokenType::Slash),
            b'"' => self.string(),
            _ => {
                if ch.is_ascii_digit() {
                    self.number()
                } else if ch.is_ascii_alphabetic() || ch == b'_' {
                    self.identifier()
                } else {
                    self.error_token("Unexpected character.")
                }
            }
        }
    }

    fn skip_whitespace(&mut self) {
        loop {
            match self.peek() {
                b' ' | b'\r' | b'\t' => {
                    self.advance();
                }
                b'\n' => {
                    self.line += 1;
                    self.advance();
                }
                b'/' if self.peek_next() == b'/' => {
                    while self.peek() != b'\n' && !self.is_at_end() {
                        self.advance();
                    }
                }
                _ => {
                    return;
                }
            }
        }
    }

    fn error_token(&mut self, msg: impl ToString) -> Token {
        Token::new(TokenType::Error, msg.to_string(), self.line)
    }

    fn identifier(&mut self) -> Token {
        while !self.is_at_end() && (self.peek().is_ascii_alphanumeric() || self.peek() == b'_') {
            self.advance();
        }
        let s = self.source[self.start..self.current].to_string();
        let token_type = TokenType::parse(&s);
        self.make_token(token_type)
    }

    fn number(&mut self) -> Token {
        while !self.is_at_end() && self.peek().is_ascii_digit() {
            self.advance();
        }
        if self.peek() == b'.' && self.peek_next().is_ascii_digit() {
            self.advance();
            while !self.is_at_end() && self.peek().is_ascii_digit() {
                self.advance();
            }
        }
        self.make_token(TokenType::Number)
    }

    fn string(&mut self) -> Token {
        while self.peek() != b'"' && !self.is_at_end() {
            if self.peek() == b'\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            return self.error_token("Unterminated string.");
        }

        let value = self.source[self.start + 1..self.current].to_string();
        self.advance();
        Token::new(TokenType::String, value, self.line)
    }

    fn make_token(&mut self, token_type: TokenType) -> Token {
        Token::new(
            token_type,
            self.source[self.start..self.current].to_string(),
            self.line,
        )
    }

    fn advance(&mut self) -> u8 {
        let c = self.source.as_bytes()[self.current];
        self.current += 1;
        c
    }

    fn peek_next(&self) -> u8 {
        if self.current + 1 >= self.source.len() {
            b'\0'
        } else {
            self.source.as_bytes()[self.current + 1]
        }
    }

    fn peek(&self) -> u8 {
        if self.is_at_end() {
            b'\0'
        } else {
            self.source.as_bytes()[self.current]
        }
    }

    fn matches(&mut self, ch: u8) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.source.as_bytes()[self.current] != ch {
            return false;
        }
        self.current += 1;
        true
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }
}
