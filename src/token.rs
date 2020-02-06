use std::io::BufReader;
use std::io::Read;
use std::ops::Index;
use std::str::Chars;
use thiserror::Error;

#[derive(Debug, PartialEq)]
enum Token {
    Illegal,
    Eof,

    // Identifiers + literals
    Ident(String),
    Int(i64),

    // Operators
    Assign,
    Plus,
    Minus,
    Bang,
    Asterisk,
    Slash,
    Eq,
    NotEq,
    Gt,
    Lt,

    // Delimiters
    Comma,
    Semicolon,
    LParen,
    RParen,
    LBrace,
    RBrace,

    // keywords
    Function,
    Let,
    Return,
    If,
    Else,
    True,
    False,
}

struct Lexer {
    input: Vec<char>,
    pos: usize,
    read_pos: usize,
    // None here means Eof
    ch: Option<char>,
}

impl Lexer {
    pub fn new<R: Read>(mut input: BufReader<R>) -> Result<Self, LexerError> {
        let mut s = String::new();
        input.read_to_string(&mut s).or(Err(LexerError::Invalid))?;
        let chars: Vec<char> = s.chars().collect();
        let mut l = Lexer {
            input: chars,
            pos: 0,
            read_pos: 0,
            ch: None,
        };
        l.read_char();
        Ok(l)
    }

    pub fn next_token(&mut self) -> Token {
        self.eat_whitespace();

        match self.ch {
            None => Token::Eof,
            Some('+') => Token::Plus,
            Some('-') => Token::Minus,
            Some('*') => Token::Asterisk,
            Some('/') => Token::Slash,
            Some(';') => Token::Semicolon,
            Some('=') => {
                match self.peek_char() {
                    Some(x) if x == '=' => Token::Eq,
                    _ => Token::Assign,
                }
            },
            Some('>') => Token::Gt,
            Some('<') => Token::Lt,
            Some('{') => Token::LBrace,
            Some('}') => Token::RBrace,
            Some('(') => Token::LParen,
            Some(')') => Token::RParen,
            // numbers, keywords, and identifiers
            Some(x) if x.is_alphabetic() => {
                let ident = self.read_identifier();
                Token::Ident(ident)
            },
            _ => Token::Illegal,
        }
    }

    fn eat_whitespace(&mut self) {
        loop {
            self.read_char();
            if self.ch.is_none() {
                break;
            }
            // unwrap is safe because we checked for None
            if !self.ch.unwrap().is_whitespace() {
                break;
            }
        }
    }

    pub fn read_char(&mut self) {
        if self.read_pos >= self.input.len() {
            self.ch = None;
        } else {
            self.ch = Some(self.input[self.pos]);
        }
        self.pos = self.read_pos;
        self.read_pos += 1;
    }

    fn read_identifier(&mut self) -> String {
        let pos = self.pos;
        self.read_char();
        while let Some(x) = self.ch {
            if !x.is_alphabetic() {
                break;
            }
            self.read_char();
        }
        use std::iter::FromIterator;
        String::from_iter(&self.input[pos..self.pos])
    }

    fn peek_char(&mut self) -> Option<char> {
        if self.read_pos >= self.input.len() {
           return None;
        } 
        return Some(self.input[self.read_pos]);
    }

}

#[derive(Debug, Error)]
enum LexerError {
    #[error("invalid input")]
    Invalid,
    #[error("empty input")]
    Empty,
}

#[test]
fn test_lexer() {
    let input = r#"let five = 5;
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
    "#;

    let br = BufReader::new(input.as_bytes());
    let mut l = Lexer::new(br).unwrap();

    assert_eq!(Token::Let, l.next_token());
    assert_eq!(Token::Ident(String::from("five")), l.next_token());
    assert_eq!(Token::Assign, l.next_token());
    assert_eq!(Token::Int(5), l.next_token());
    assert_eq!(Token::Semicolon, l.next_token());
    assert_eq!(Token::Let, l.next_token());
    assert_eq!(Token::Ident(String::from("ten")), l.next_token());
    assert_eq!(Token::Assign, l.next_token());
    assert_eq!(Token::Int(10), l.next_token());
    assert_eq!(Token::Semicolon, l.next_token());
    assert_eq!(Token::Let, l.next_token());
    assert_eq!(Token::Ident(String::from("add")), l.next_token());
    assert_eq!(Token::Assign, l.next_token());
    assert_eq!(Token::Function, l.next_token());
    assert_eq!(Token::LParen, l.next_token());
    assert_eq!(Token::Ident(String::from("x")), l.next_token());
    assert_eq!(Token::Comma, l.next_token());
    assert_eq!(Token::Ident(String::from("y")), l.next_token());
    assert_eq!(Token::RParen, l.next_token());
    assert_eq!(Token::LBrace, l.next_token());
    assert_eq!(Token::Ident(String::from("x")), l.next_token());
    assert_eq!(Token::Plus, l.next_token());
    assert_eq!(Token::Ident(String::from("y")), l.next_token());
    assert_eq!(Token::Semicolon, l.next_token());
    assert_eq!(Token::RBrace, l.next_token());
    assert_eq!(Token::Semicolon, l.next_token());
    assert_eq!(Token::Let, l.next_token());
    assert_eq!(Token::Ident(String::from("result")), l.next_token());
    assert_eq!(Token::Assign, l.next_token());
    assert_eq!(Token::Ident(String::from("add")), l.next_token());
    assert_eq!(Token::LParen, l.next_token());
    assert_eq!(Token::Ident(String::from("five")), l.next_token());
    assert_eq!(Token::Comma, l.next_token());
    assert_eq!(Token::Ident(String::from("ten")), l.next_token());
    assert_eq!(Token::RParen, l.next_token());
    assert_eq!(Token::Semicolon, l.next_token());
    assert_eq!(Token::Bang, l.next_token());
    assert_eq!(Token::Minus, l.next_token());
    assert_eq!(Token::Slash, l.next_token());
    assert_eq!(Token::Asterisk, l.next_token());
    assert_eq!(Token::Int(5), l.next_token());
    assert_eq!(Token::Semicolon, l.next_token());
    assert_eq!(Token::Int(5), l.next_token());
    assert_eq!(Token::Lt, l.next_token());
    assert_eq!(Token::Int(10), l.next_token());
    assert_eq!(Token::Gt, l.next_token());
    assert_eq!(Token::Int(5), l.next_token());
    assert_eq!(Token::Semicolon, l.next_token());
    assert_eq!(Token::If, l.next_token());
    assert_eq!(Token::LParen, l.next_token());
    assert_eq!(Token::Int(5), l.next_token());
    assert_eq!(Token::Lt, l.next_token());
    assert_eq!(Token::Int(10), l.next_token());
    assert_eq!(Token::RParen, l.next_token());
    assert_eq!(Token::LBrace, l.next_token());
    assert_eq!(Token::Return, l.next_token());
    assert_eq!(Token::True, l.next_token());
    assert_eq!(Token::Semicolon, l.next_token());
    assert_eq!(Token::RBrace, l.next_token());
    assert_eq!(Token::Else, l.next_token());
    assert_eq!(Token::LBrace, l.next_token());
    assert_eq!(Token::Return, l.next_token());
    assert_eq!(Token::False, l.next_token());
    assert_eq!(Token::Semicolon, l.next_token());
    assert_eq!(Token::RBrace, l.next_token());
    assert_eq!(Token::Int(10), l.next_token());
    assert_eq!(Token::Eq, l.next_token());
    assert_eq!(Token::Int(10), l.next_token());
    assert_eq!(Token::Semicolon, l.next_token());
    assert_eq!(Token::Int(10), l.next_token());
    assert_eq!(Token::NotEq, l.next_token());
    assert_eq!(Token::Int(9), l.next_token());
    assert_eq!(Token::Semicolon, l.next_token());
    assert_eq!(Token::Eof, l.next_token());
}

/*
    tests := []struct {
        expectedType    token.TokenType
        expectedLiteral string
    }{
        assert_eq!(Token::Let, l.next_token()),
        assert_eq!(Token::Ident(String::from("five")), l.next_token()),
        assert_eq!(Token::Assign, l.next_token()),
        assert_eq!(Token::Int(5), l.next_token()),
        assert_eq!(Token::Semicolon, l.next_token()),
        assert_eq!(Token::Let, l.next_token()),
        assert_eq!(Token::Ident(String::from("ten")), l.next_token()),
        assert_eq!(Token::Assign, l.next_token()),
        assert_eq!(Token::Int(10), l.next_token()),
        assert_eq!(Token::Semicolon, l.next_token()),
        assert_eq!(Token::Let, l.next_token()),
        assert_eq!(Token::Ident(String::from("add")), l.next_token()),
        assert_eq!(Token::Assign, l.next_token()),
        assert_eq!(Token::Function, l.next_token()),
        assert_eq!(Token::LParen, l.next_token()),
        assert_eq!(Token::Ident(String::from("x")), l.next_token()),
        assert_eq!(Token::Comma, l.next_token()),
        assert_eq!(Token::Ident(String::from("y")), l.next_token()),
        assert_eq!(Token::RParen, l.next_token()),
        assert_eq!(Token::LBrace, l.next_token()),
        assert_eq!(Token::Ident, l.next_token()),
        assert_eq!(Token::Plus, l.next_token()),
        assert_eq!(Token::Ident(String::from("y")), l.next_token()),
        assert_eq!(Token::Semicolon, l.next_token()),
        assert_eq!(Token::RBrace, l.next_token()),
        assert_eq!(Token::Semicolon, l.next_token()),
        assert_eq!(Token::Let, l.next_token()),
        assert_eq!(Token::Ident(String::from("result")), l.next_token()),
        assert_eq!(Token::Assign, l.next_token()),
        assert_eq!(Token::Ident(String::from("add")), l.next_token()),
        assert_eq!(Token::LParen, l.next_token()),
        assert_eq!(Token::Ident(String::from("five")), l.next_token()),
        assert_eq!(Token::Comma, l.next_token()),
        assert_eq!(Token::Ident(String::from("ten")), l.next_token()),
        assert_eq!(Token::RParen, l.next_token()),
        assert_eq!(Token::Semicolon, l.next_token()),
        assert_eq!(Token::Bang, l.next_token()),
        assert_eq!(Token::Minus, l.next_token()),
        assert_eq!(Token::Slash, l.next_token()),
        assert_eq!(Token::Asterisk, l.next_token()),
        assert_eq!(Token::Int(5), l.next_token()),
        assert_eq!(Token::Semicolon, l.next_token()),
        assert_eq!(Token::Int(5), l.next_token()),
        assert_eq!(Token::Lt, l.next_token()),
        assert_eq!(Token::Int(10), l.next_token()),
        assert_eq!(Token::Gt, l.next_token()),
        assert_eq!(Token::Int(5), l.next_token()),
        assert_eq!(Token::Semicolon, l.next_token()),
        assert_eq!(Token::If, l.next_token()),
        assert_eq!(Token::LParen, l.next_token()),
        assert_eq!(Token::Int(5), l.next_token()),
        assert_eq!(Token::Lt, l.next_token()),
        assert_eq!(Token::Int(10), l.next_token()),
        assert_eq!(Token::RParen, l.next_token()),
        assert_eq!(Token::LBrace, l.next_token()),
        assert_eq!(Token::Return, l.next_token()),
        assert_eq!(Token::True, l.next_token()),
        assert_eq!(Token::Semicolon, l.next_token()),
        assert_eq!(Token::RBrace, l.next_token()),
        assert_eq!(Token::Else, l.next_token()),
        assert_eq!(Token::LBrace, l.next_token()),
        assert_eq!(Token::Return, l.next_token()),
        assert_eq!(Token::False, l.next_token()),
        assert_eq!(Token::Semicolon, l.next_token()),
        assert_eq!(Token::RBrace, l.next_token()),
        assert_eq!(Token::Int(10), l.next_token()),
        assert_eq!(Token::Eq, l.next_token()),
        assert_eq!(Token::Int(10), l.next_token()),
        assert_eq!(Token::Semicolon, l.next_token()),
        assert_eq!(Token::Int(10), l.next_token()),
        assert_eq!(Token::NotEq, l.next_token()),
        assert_eq!(Token::Int(9), l.next_token()),
        assert_eq!(Token::Semicolon, l.next_token()),
        assert_eq!(Token::Eof, l.next_token()),
    )

    l := New(input)

    for i, tt := range tests {
        tok := l.NextToken()

        if tok.Type != tt.expectedType {
            t.Fatalf("tests[%d] - tokentype wrong. expected=%q, got=%q",
                i, tt.expectedType, tok.Type)
        }

        if tok.Literal != tt.expectedLiteral {
            t.Fatalf("tests[%d] - literal wrong. expected=%q, got=%q",
                i, tt.expectedLiteral, tok.Literal)
        }
    }
}

*/
