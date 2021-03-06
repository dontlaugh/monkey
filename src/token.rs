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
        // println!("CURRENT CHAR {:?}", self.ch);

        let tkn = match self.ch {
            None => Token::Eof,
            Some(',') => Token::Comma,
            Some('+') => Token::Plus,
            Some('-') => Token::Minus,
            Some('*') => Token::Asterisk,
            Some('/') => Token::Slash,
            Some(';') => Token::Semicolon,
            Some('=') => match self.peek_char() {
                Some(x) if x == '=' => {
                    self.read_char();
                    Token::Eq
                },
                Some(_) => Token::Assign,
                _ => unreachable!()
            },
            Some('!') => match self.peek_char() {
                Some(x) if x == '=' => {
                    self.read_char();
                    Token::NotEq
                },
                Some(_) => Token::Bang,
                _ => unreachable!()
            },
            Some('>') => Token::Gt,
            Some('<') => Token::Lt,
            Some('{') => Token::LBrace,
            Some('}') => Token::RBrace,
            Some('(') => Token::LParen,
            Some(')') => Token::RParen,
            // numbers, keywords, and identifiers
            // We have lots of early returns here because the internal behavior
            // of read_identifier and read_int has it's own looping calls to
            // read_char. The single-char cases of this match statement (the one
            // we're in right now) do not, and instead rely on the call to 
            // read_char at the end of this function to advance our lexer.
            Some(x) if x.is_ascii_alphabetic() => {
                let ident = self.read_identifier();
                if is_keyword(&ident) {
                    match ident.as_str() {
                        "fn" => return Token::Function,
                        "let" => return Token::Let,
                        "return" => return Token::Return,
                        "if" => return Token::If,
                        "else" => return Token::Else,
                        "true" => return Token::True,
                        "false" => return Token::False,
                        _ => unreachable!(),
                    }
                } else {
                    return Token::Ident(ident)
                }
            }
            Some(x) if x.is_ascii_digit() => {
                let i = self.read_int();
                if i.is_none() {
                    // error parsing int
                    return Token::Illegal
                } else {
                    return Token::Int(i.unwrap())
                }
            }
            _ => Token::Illegal,
        };
        self.read_char();
        tkn
    }

    fn eat_whitespace(&mut self) {
        loop {
            if self.ch.is_none() {
                return;
            }
            // unwrap is safe because we checked for None
            if !self.ch.unwrap().is_whitespace() {
                return;
            }
            //println!("EAT WHITESPACE READ CHAR");
            self.read_char();
        }
    }

    pub fn read_char(&mut self) {
        if self.read_pos >= self.input.len() {
            self.ch = None;
        } else {
            self.ch = Some(self.input[self.read_pos]);
        }
        self.pos = self.read_pos;
        self.read_pos += 1;
    }

    fn read_identifier(&mut self) -> String {
        let pos = self.pos;
        while let Some(x) = self.ch {
            //println!("READIDENTIFIER: {:?} pos {:?} self.pos {:?}", x, pos, self.pos);
            if x.is_alphabetic() {
                self.read_char();
            } else {
                break;
            }
        }
        use std::iter::FromIterator;
        String::from_iter(&self.input[pos..self.pos])
    }

    fn read_int(&mut self) -> Option<i64> {
        let pos = self.pos;
        while let Some(x) = self.ch {
            //println!("READINT: {:?} pos {:?} self.pos {:?}", x, pos, self.pos);
            if x.is_ascii_digit() {
                self.read_char();
            } else {
                break;
            }
        }
        use std::iter::FromIterator;
        use std::str::FromStr;
        let s = String::from_iter(&self.input[pos..self.pos]);
        i64::from_str(&s).ok()
    }

    fn peek_char(&mut self) -> Option<char> {
        if self.read_pos >= self.input.len() {
            return None;
        }
        return Some(self.input[self.read_pos]);
    }
}

fn is_keyword(s: &str) -> bool {
    match s {
        "fn" | "let" | "return" | "if" | "else" | "true" | "false" => true,
        _ => false,
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

#[test]
fn test_read_identifier() {
    // no leading whitespace
    let input = "foo   ";
    let br = BufReader::new(input.as_bytes());
    let mut l = Lexer::new(br).unwrap();
    let output = l.read_identifier();
    assert_eq!("foo".to_owned(), output);

    // leading whitespace to eat first (yum!)
    let input = "   foo   ";
    let br = BufReader::new(input.as_bytes());
    let mut l = Lexer::new(br).unwrap();
    l.eat_whitespace();
    let output = l.read_identifier();
    assert_eq!("foo".to_owned(), output);
}


#[test]
fn test_read_int() {

    // no leading whitespace
    let input = "10   ";
    let br = BufReader::new(input.as_bytes());
    let mut l = Lexer::new(br).unwrap();
    let output = l.read_int();
    assert_eq!(Some(10), output);

    // leading whitespace to eat first (yum!)
    let input = "   7   ";
    let br = BufReader::new(input.as_bytes());
    let mut l = Lexer::new(br).unwrap();
    l.eat_whitespace();
    let output = l.read_int();
    assert_eq!(Some(7), output);

    let input = "   7;;   ";
    let br = BufReader::new(input.as_bytes());
    let mut l = Lexer::new(br).unwrap();
    l.eat_whitespace();
    let output = l.read_int();
    assert_eq!(Some(7), output);

}
