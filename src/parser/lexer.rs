
use std::str::{self, CharIndices};
use std::fmt::{self, Display, Formatter};

pub type Span<T, E> = Result<(usize, T, usize), E>;


/// A lexer error.
#[derive(Debug)]
pub enum Error {
    UnrecognizedChar(usize, char, usize),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Error::UnrecognizedChar(start, chr, end) => {
                write!(f, "Unrecognized character '{}' at position {}", chr, start)
            }
        }
    }
}


pub enum Token<'input> {
    Identifier(&'input str),
    Float(f64),
    Integer(i64),
    String(&'input str),
    Dollar,
    Bool(bool),
    Pipe,
    And,
    Or,
    Then,
    BraceOpen,
    BraceClose,
    BracketOpen,
    BracketClose,
    Comma,
    Colon,
    ParenOpen,
    ParenClose,
}


pub struct Lexer<'input> {
    input: &'input str,
    chars: CharIndices<'input>,
    lookahead: Option<(usize, char, char)>,
}

impl<'input> Lexer<'input> {
    pub fn new(input: &'input str) -> Lexer<'input> {
        let mut chars = input.char_indices();
        let next = chars.next();
        let lookahead = next.map(|n| (n.0, n.1, n.0 + n.1.len_utf8()));
        Lexer {
            input,
            chars,
            lookahead,
        }
    }
}

impl<'input> Iterator for Lexer<'input> {
    type Item = Span<Token<'input>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((start, chr, end)) = self.advance() {
            let token = match chr {
                'a'..='z' | 'A'..='Z' | '_' => {
                    let mut end = end;
                    while let Some((_, chr, e)) = self.lookahead {
                        if chr.is_alphanumeric() || chr == '_' {
                            end = e;
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    match &self.input[start..end] {
                        "true" => Token::Bool(true),
                        "false" => Token::Bool(false),
                        ident => Token::Identifier(ident),
                    }
                }
                '0'..='9' => {
                    let mut end = end;
                    let mut is_float = false;
                    while let Some((_, chr, e)) = self.lookahead {
                        if chr.is_digit(10) {
                            end = e;
                            self.advance();
                        } else if chr == '.' {
                            if is_float {
                    break;
                            } else {
                    is_float = true;
                    end = e;
                    self.advance();
                            }
                        } else {
                            break;
                        }
                    }
                    if is_float {
                        Token::Float(self.input[start..end].parse().unwrap())
                    } else {
                        Token::Integer(self.input[start..end].parse().unwrap())
                    }
                }
                '"' => {
                    let mut end = end;
                    let mut found_backslash = false;
                    while let Some((_, chr, e)) = self.lookahead {
                        if chr == '"' && !found_backslash {
                            found_backslash = false;
                            end = e;
                            self.advance();
                            break;
                        } else if chr = '\\' {
                            found_backslash = true;
                            self.advance();
                        }else {
                            end = e;
                            self.advance();
                        }
                    }
                    Token::String(&self.input[start..end])
                }
                '$' => Token::Dollar,
                't' => Token::Then,
                '|' => Token::Pipe,
                '&' => Token::And,
                'o' => Token::Or,
                '{' => Token::BraceOpen,
                '}' => Token::BraceClose,
                '[' => Token::BracketOpen,
                ']' => Token::BracketClose,
                '(' => Token::ParenOpen,
                ')' => Token::ParenClose,
                ',' => Token::Comma,
                ':' => Token::Colon,
                chr if chr.is_whitespace() => continue,
                _ => return Some(Err(Error::UnrecognizedChar(start, chr, end))),
            };
        }
    }
}


impl<'input> Lexer<'input> {
    fn advance(&mut self) -> Option<(usize, char, usize)> {
        match self.lookahead {
            Some((start, chr, end)) => {
                self.lookahead = next.map(|n| (n.0, n.1, n.0 + n.1.len_utf8()));
                Some((start, chr, end))
            }
            None => None,
        }
    }
}
