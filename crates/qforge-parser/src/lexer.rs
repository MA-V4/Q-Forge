// OpenQASM 2.0 Lexer
// Phase 1 deliverable: tokenise a .qasm file into a token stream.

use crate::error::ParseError;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Keywords
    OpenQasm,
    Include,
    Qreg,
    Creg,
    Barrier,
    Measure,
    Reset,
    If,
    Gate,
    Opaque,
    // Punctuation
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    Semicolon,
    Comma,
    Arrow,  // ->
    Equals, // ==
    Plus,
    Minus,
    Star,
    Slash,
    Caret,
    // Literals
    Ident(String),
    Integer(i64),
    Real(f64),
    StringLit(String),
    // End
    Eof,
}

#[allow(dead_code)]
pub struct Lexer {
    input: Vec<char>,
    pos: usize,
    pub line: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            pos: 0,
            line: 1,
        }
    }

    fn peek(&self) -> Option<char> {
        self.input.get(self.pos).copied()
    }
    fn peek2(&self) -> Option<char> {
        self.input.get(self.pos + 1).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.input.get(self.pos).copied();
        if ch == Some('\n') {
            self.line += 1;
        }
        if self.pos < self.input.len() {
            self.pos += 1;
        }
        ch
    }

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            match self.peek() {
                Some(' ') | Some('\t') | Some('\r') | Some('\n') => {
                    self.advance();
                }
                Some('/') if self.peek2() == Some('/') => {
                    while let Some(c) = self.advance() {
                        if c == '\n' {
                            break;
                        }
                    }
                }
                _ => break,
            }
        }
    }

    fn read_string(&mut self) -> Result<String, ParseError> {
        self.advance(); // opening "
        let mut s = String::new();
        loop {
            match self.advance() {
                None | Some('\n') => {
                    return Err(ParseError::UnexpectedToken {
                        line: self.line,
                        message: "unterminated string".into(),
                    })
                }
                Some('"') => break,
                Some(c) => s.push(c),
            }
        }
        Ok(s)
    }

    fn read_number(&mut self) -> Token {
        let mut s = String::new();
        let mut is_real = false;
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                s.push(c);
                self.advance();
            } else if c == '.' && !is_real {
                s.push(c);
                is_real = true;
                self.advance();
            } else if (c == 'e' || c == 'E') && !s.contains('e') {
                s.push(c);
                is_real = true;
                self.advance();
                if self.peek() == Some('-') || self.peek() == Some('+') {
                    s.push(self.advance().unwrap());
                }
            } else {
                break;
            }
        }
        if is_real {
            Token::Real(s.parse().unwrap_or(0.0))
        } else {
            Token::Integer(s.parse().unwrap_or(0))
        }
    }

    fn read_ident(&mut self) -> String {
        let mut s = String::new();
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' {
                s.push(c);
                self.advance();
            } else {
                break;
            }
        }
        s
    }

    fn keyword_or_ident(s: String) -> Token {
        match s.as_str() {
            "OPENQASM" => Token::OpenQasm,
            "include" => Token::Include,
            "qreg" => Token::Qreg,
            "creg" => Token::Creg,
            "barrier" => Token::Barrier,
            "measure" => Token::Measure,
            "reset" => Token::Reset,
            "if" => Token::If,
            "gate" => Token::Gate,
            "opaque" => Token::Opaque,
            "pi" => Token::Real(std::f64::consts::PI),
            _ => Token::Ident(s),
        }
    }

    pub fn tokenise(&mut self) -> Result<Vec<Token>, ParseError> {
        let mut out = Vec::new();
        loop {
            self.skip_whitespace_and_comments();
            let line = self.line;
            match self.peek() {
                None => {
                    out.push(Token::Eof);
                    break;
                }
                Some('(') => {
                    self.advance();
                    out.push(Token::LParen);
                }
                Some(')') => {
                    self.advance();
                    out.push(Token::RParen);
                }
                Some('[') => {
                    self.advance();
                    out.push(Token::LBracket);
                }
                Some(']') => {
                    self.advance();
                    out.push(Token::RBracket);
                }
                Some('{') => {
                    self.advance();
                    out.push(Token::LBrace);
                }
                Some('}') => {
                    self.advance();
                    out.push(Token::RBrace);
                }
                Some(';') => {
                    self.advance();
                    out.push(Token::Semicolon);
                }
                Some(',') => {
                    self.advance();
                    out.push(Token::Comma);
                }
                Some('+') => {
                    self.advance();
                    out.push(Token::Plus);
                }
                Some('*') => {
                    self.advance();
                    out.push(Token::Star);
                }
                Some('^') => {
                    self.advance();
                    out.push(Token::Caret);
                }
                Some('/') => {
                    self.advance();
                    out.push(Token::Slash);
                }
                Some('-') => {
                    self.advance();
                    if self.peek() == Some('>') {
                        self.advance();
                        out.push(Token::Arrow);
                    } else {
                        out.push(Token::Minus);
                    }
                }
                Some('=') => {
                    self.advance();
                    if self.peek() == Some('=') {
                        self.advance();
                        out.push(Token::Equals);
                    } else {
                        return Err(ParseError::UnexpectedToken {
                            line,
                            message: "expected '==' not '='".into(),
                        });
                    }
                }
                Some('"') => {
                    let s = self.read_string()?;
                    out.push(Token::StringLit(s));
                }
                Some(c) if c.is_ascii_digit() => {
                    out.push(self.read_number());
                }
                Some(c) if c.is_alphabetic() || c == '_' => {
                    let ident = self.read_ident();
                    out.push(Self::keyword_or_ident(ident));
                }
                Some(c) => {
                    let ch = c;
                    self.advance();
                    return Err(ParseError::UnexpectedToken {
                        line,
                        message: format!("unexpected character '{}'", ch),
                    });
                }
            }
        }
        Ok(out)
    }
}
