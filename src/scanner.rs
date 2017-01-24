use std::str::Chars;
use std::iter::Peekable;
use result::{Result, Error};
use token::{Token, Type, Literal};
use std::collections::{HashSet, VecDeque};
use std::ops::Index;

pub struct Scanner<'a> {
    src: Chars<'a>,
    peeks: VecDeque<char>,
    lexeme: String,
    line: u64,
    eof: bool,
}

fn new(c: Chars) -> Scanner {
    Scanner {
        src: c,
        peeks: VecDeque::with_capacity(2),
        lexeme: "".to_string(),
        line: 1,
        eof: false,
    }
}

impl<'a> Scanner<'a> {
    fn advance(&mut self) -> Option<char> {
        if self.eof {
            return None
        }

        match self.peeks.len() {
            0 => self.src.next(),
            _ => self.peeks.pop_front(),
        }.or_else(|| {
            self.eof = true;
            Some('\0')
        }).and_then(|c| {
            self.lexeme.push(c);
            Some(c)
        })
    }

    fn lookahead(&mut self, n: usize) -> char {
        assert!(n > 0, "lookahead must be greater than zero");

        while self.peeks.len() < n {
            self.src.next().
                or(Some('\0')).
                map(|c| self.peeks.push_back(c));
        }

        *self.peeks.index(n - 1)
    }

    fn peek(&mut self) -> char {
        self.lookahead(1)
    }

    fn peek_next(&mut self) -> char {
        self.lookahead(2)
    }

    fn match_advance(&mut self, c: char) -> bool {
        if self.peek() == c {
            self.advance().unwrap();
            return true
        }

        false
    }

    fn advance_until(&mut self, c: HashSet<char>) -> char {
        let mut last = '\0';

        loop {
            match self.peek() {
                ch if c.contains(&ch) || ch == '\0' => break,
                ch => {
                    last = ch;
                    self.advance()
                },
            };
        };
        last
    }
}

impl<'a> Scanner<'a> {
    fn static_token(&self, typ: Type) -> Option<Result<Token>> {
        self.literal_token(typ, None)
    }

    fn literal_token(&self, typ: Type, lit: Option<Literal>) -> Option<Result<Token>> {
        Some(Ok(Token {
            typ: typ,
            literal: lit,
            line: self.line,
            lexeme: self.lexeme.clone(),
        }))
    }

    fn err(&self, msg: &str) -> Option<Result<Token>> {
        Some(Err(Error::Lexical(self.line, msg.to_string(), self.lexeme.clone()).boxed()))
    }

    fn match_static_token(&mut self, c: char, m: Type, u: Type) -> Option<Result<Token>> {
        match self.match_advance(c) {
            true => self.static_token(m),
            false => self.static_token(u),
        }
    }

    fn string(&mut self) -> Option<Result<Token>> {
        loop {
            let last = self.advance_until(['\n', '"'].iter().cloned().collect());

            match self.peek() {
                '\n' => self.line += 1,
                '"' if last == '\\' => { self.lexeme.pop(); },
                '"' => break,
                '\0' => return self.err("unterminated string"),
                _ => return self.err("unexpected character"),
            };

            self.advance();
        }

        self.advance();

        let lit: String = self.lexeme.clone()
            .chars()
            .skip(1)
            .take(self.lexeme.len() - 2)
            .collect();

        self.literal_token(Type::String, Some(Literal::String(lit)))
    }

    fn number(&mut self) -> Option<Result<Token>> {
        while self.peek().is_digit(10) { self.advance(); };

        if self.peek() == '.' && self.peek_next().is_digit(10) {
            self.advance();
            while self.peek().is_digit(10) { self.advance(); };
        }

        if let Ok(lit) = self.lexeme.clone().parse::<f64>() {
            return self.literal_token(Type::Number, Some(Literal::Number(lit)));
        }

        self.err("invalid numeric")
    }
}

impl<'a> Iterator for Scanner<'a> {
    type Item = Result<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        use token::Type::*;

        if self.eof {
            return None
        }

        self.lexeme.clear();

        loop {
            match self.advance().unwrap() {
                '\0' => return self.static_token(EOF),

                '(' => return self.static_token(LeftParen),
                ')' => return self.static_token(RightParen),
                '{' => return self.static_token(LeftBrace),
                '}' => return self.static_token(RightBrace),
                ',' => return self.static_token(Comma),
                '.' => return self.static_token(Dot),
                '-' => return self.static_token(Minus),
                '+' => return self.static_token(Plus),
                ';' => return self.static_token(Semicolon),
                '*' => return self.static_token(Star),

                '!' => return self.match_static_token('=', BangEqual, Bang),
                '=' => return self.match_static_token('=', EqualEqual, Equal),
                '<' => return self.match_static_token('=', LessEqual, Less),
                '>' => return self.match_static_token('=', GreaterEqual, Greater),

                '"' => return self.string(),
                c if c.is_digit(10) => return self.number(),

                '/' => {
                    if self.match_advance('/') {
                        self.advance_until(['\n'].iter().cloned().collect());
                        self.lexeme.clear();
                    } else {
                        return self.static_token(Slash);
                    }
                },

                c if c.is_whitespace() => {
                    self.lexeme.clear();
                    if c == '\n' {
                        self.line += 1
                    }
                },

                _ => return self.err("unexpected character"),
            }
        }
    }
}

pub trait TokenIterator<'a> {
    fn tokens(self) -> Scanner<'a>;
}

impl<'a> TokenIterator<'a> for Chars<'a> {
    fn tokens(self) -> Scanner<'a> {
        new(self)
    }
}
