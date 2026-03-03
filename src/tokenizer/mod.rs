pub mod characters;

use self::characters::*;
use crate::ast::Token;

const EOF_CHAR: char = '\0';

pub struct Tokenizer<'a> {
    src: &'a str,
    pos: usize,
}

impl<'a> Tokenizer<'a> {
    pub fn new(src: &'a str) -> Self {
        Tokenizer { src, pos: 0 }
    }

    pub fn pos(&self) -> usize {
        self.pos
    }

    #[inline]
    fn peek(&self) -> char {
        self.src[self.pos..].chars().next().unwrap_or(EOF_CHAR)
    }

    #[inline]
    fn peek_at(&self, offset: usize) -> char {
        // offset is in characters, not bytes
        self.src[self.pos..].chars().nth(offset).unwrap_or(EOF_CHAR)
    }

    #[inline]
    fn advance(&mut self) {
        if let Some(ch) = self.src[self.pos..].chars().next() {
            self.pos += ch.len_utf8();
        }
    }

    #[inline]
    fn slice(&self, start: usize, end: usize) -> &'a str {
        &self.src[start..end]
    }

    pub fn scan(&mut self) -> (Token, &'a str, usize) {
        let pos = self.pos;
        let ch = self.peek();

        if ch == EOF_CHAR {
            return (Token::EOF, "", pos);
        }

        if is_whitespace(ch) {
            return self.scan_whitespace();
        }

        if ch == ':' {
            self.advance();
            if self.peek() == ':' {
                self.advance();
                return (Token::DoubleColon, self.slice(pos, self.pos), pos);
            }
            return (Token::Colon, self.slice(pos, self.pos), pos);
        }

        if ch == '/' {
            self.advance();
            if self.peek() == '/' {
                self.advance();
                // Consume until end of line
                while self.peek() != EOF_CHAR && self.peek() != '\n' {
                    self.advance();
                }
                if self.peek() == '\n' {
                    self.advance();
                }
                return self.scan();
            } else {
                return (Token::Slash, self.slice(pos, self.pos), pos);
            }
        }

        if is_leading_identifier_character(ch) {
            return self.scan_ident();
        }

        if is_operator_character(ch) {
            return self.scan_operator();
        }

        if ch == '.' {
            return self.scan_dots();
        }

        if ch == '\'' || ch == '"' {
            return self.scan_string(ch);
        }

        if is_digit(ch) {
            return self.scan_number();
        }

        self.advance();
        let lit = self.slice(pos, self.pos);
        let tok = match ch {
            ',' => Token::Comma,
            '(' => Token::ParenLeft,
            ')' => Token::ParenRight,
            '{' => Token::BraceLeft,
            '}' => Token::BraceRight,
            '[' => Token::BracketLeft,
            ']' => Token::BracketRight,
            '^' => Token::Hat,
            '*' => Token::Asterisk,
            '@' => Token::At,
            '|' => Token::Pipe,
            ';' => Token::Semicolon,
            _ => Token::Illegal,
        };
        (tok, lit, pos)
    }

    fn scan_whitespace(&mut self) -> (Token, &'a str, usize) {
        let pos = self.pos;
        while is_whitespace(self.peek()) {
            self.advance();
        }
        (Token::Whitespace, self.slice(pos, self.pos), pos)
    }

    fn scan_ident(&mut self) -> (Token, &'a str, usize) {
        let pos = self.pos;
        self.advance(); // consume first char

        while is_identifier_character(self.peek()) {
            self.advance();
        }

        let lit = self.slice(pos, self.pos);
        let tok = match lit {
            "true" | "false" => Token::Bool,
            "null" => Token::Null,
            "match" => Token::MatchOperator,
            "in" => Token::InOperator,
            "asc" => Token::AscOperator,
            "desc" => Token::DescOperator,
            _ => Token::Name,
        };
        (tok, lit, pos)
    }

    fn scan_operator(&mut self) -> (Token, &'a str, usize) {
        let pos = self.pos;
        let ch1 = self.peek();
        self.advance();
        let ch2 = self.peek();

        if is_operator_character(ch2) {
            let two_char = (ch1, ch2);
            let tok = match two_char {
                ('=', '=') => Some(Token::Equals),
                ('|', '|') => Some(Token::Or),
                ('&', '&') => Some(Token::And),
                ('<', '=') => Some(Token::LTE),
                ('>', '=') => Some(Token::GTE),
                ('=', '>') => Some(Token::Rocket),
                ('!', '=') => Some(Token::NEQ),
                ('-', '>') => Some(Token::Arrow),
                ('*', '*') => Some(Token::Exponentiation),
                _ => None,
            };
            if let Some(t) = tok {
                self.advance();
                return (t, self.slice(pos, self.pos), pos);
            }
        }

        let tok = match ch1 {
            '>' => Token::GT,
            '<' => Token::LT,
            '!' => Token::Not,
            '+' => Token::Plus,
            '-' => Token::Minus,
            '*' => Token::Asterisk,
            '/' => Token::Slash,
            '%' => Token::Percent,
            '|' => Token::Pipe,
            '=' => Token::EqualSign,
            _ => Token::Illegal,
        };
        (tok, self.slice(pos, self.pos), pos)
    }

    fn scan_dots(&mut self) -> (Token, &'a str, usize) {
        let pos = self.pos;
        while self.peek() == '.' {
            self.advance();
        }
        let lit = self.slice(pos, self.pos);
        let tok = match lit {
            "." => Token::Dot,
            ".." => Token::DotDot,
            "..." => Token::DotDotDot,
            _ => Token::Illegal,
        };
        (tok, lit, pos)
    }

    fn scan_number(&mut self) -> (Token, &'a str, usize) {
        let pos = self.pos;
        let mut decimal_point_seen = false;
        let mut exponent_seen = false;
        let mut last_was_digit = false;
        let mut last_was_exponent = false;

        loop {
            let ch = self.peek();
            match ch {
                EOF_CHAR => break,
                '0'..='9' => {
                    self.advance();
                    last_was_digit = true;
                    last_was_exponent = false;
                }
                'e' | 'E' if !exponent_seen => {
                    self.advance();
                    exponent_seen = true;
                    last_was_digit = false;
                    last_was_exponent = true;
                }
                '+' | '-' if last_was_exponent => {
                    self.advance();
                    last_was_digit = false;
                    last_was_exponent = false;
                }
                '.' if !decimal_point_seen && !exponent_seen => {
                    if self.peek_at(1) == '.' {
                        break; // Range operator
                    }
                    self.advance();
                    decimal_point_seen = true;
                    last_was_digit = false;
                    last_was_exponent = false;
                }
                _ => break,
            }
        }

        let lit = self.slice(pos, self.pos);
        if !last_was_digit {
            return (Token::Illegal, lit, pos);
        }

        let tok = if decimal_point_seen || exponent_seen {
            Token::Float
        } else {
            Token::Integer
        };
        (tok, lit, pos)
    }

    fn scan_string(&mut self, quote: char) -> (Token, &'a str, usize) {
        let pos = self.pos;
        self.advance(); // consume opening quote

        loop {
            let ch = self.peek();
            if ch == EOF_CHAR {
                return (Token::Illegal, self.slice(pos, self.pos), pos);
            }

            if ch == '\\' {
                self.advance();
                let escaped = self.peek();
                if escaped == EOF_CHAR {
                    return (Token::Illegal, self.slice(pos, self.pos), pos);
                }
                // Handle \uXXXX - need to skip 4 hex digits
                if escaped == 'u' {
                    self.advance();
                    if self.peek() == '{' {
                        // \u{XXXX} format
                        self.advance();
                        while self.peek().is_ascii_hexdigit() {
                            self.advance();
                        }
                        if self.peek() == '}' {
                            self.advance();
                        }
                    } else {
                        // \uXXXX format - consume 4 hex digits
                        for _ in 0..4 {
                            if self.peek().is_ascii_hexdigit() {
                                self.advance();
                            }
                        }
                        // Check for surrogate pair
                        if self.peek() == '\\' && self.peek_at(1) == 'u' {
                            // Might be a surrogate pair, continue
                        }
                    }
                } else {
                    self.advance();
                }
                continue;
            }

            if ch == quote {
                self.advance();
                return (Token::String, self.slice(pos, self.pos), pos);
            }

            self.advance();
        }
    }
}
