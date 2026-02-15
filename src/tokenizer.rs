use core::panic;

use crate::span::*;

#[derive(Debug, Clone, PartialEq)]
pub enum LitKind {
    Integer,
    Str,
}

pub type Symbol = String;

#[derive(Debug, Clone, PartialEq)]
pub struct Lit {
    pub kind: LitKind,
    pub symbol: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum KeywordKind {
    Return,
    If,
    Else,
    While,
    For,
}

#[derive(Debug, PartialEq, Clone)]
pub enum TokenKind {
    Keyword(KeywordKind),
    Literal(Lit),
    Ident(Symbol),
    Add,
    Sub,
    Mul,
    Div,
    LParen,
    RParen,
    LBrace,
    RBrace,
    Eq,
    EqEq,
    Ne,
    Ge,
    Gt,
    Le,
    Lt,
    Semi,
    Comma,
    Reserved(String),
    Eof,
}

impl KeywordKind {
    pub fn lex_keyword(token: &str) -> Option<KeywordKind> {
        match token {
            "return" => Some(KeywordKind::Return),
            "if" => Some(KeywordKind::If),
            "else" => Some(KeywordKind::Else),
            "for" => Some(KeywordKind::For),
            "while" => Some(KeywordKind::While),
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

pub type TokenContainer = Vec<Token>;

pub fn parse_next_number(s: &[u8], cursor: &mut usize) -> String {
    let start = *cursor;
    while let Some(c) = s.get(*cursor) {
        if !c.is_ascii_digit() {
            break;
        }
        *cursor += 1;
    }
    String::from_utf8(s[start..*cursor].to_vec()).unwrap()
}

pub fn parse_next_ident(s: &[u8], cursor: &mut usize) -> String {
    let start = *cursor;
    while let Some(c) = s.get(*cursor) {
        if !(c.is_ascii_digit() | c.is_ascii_alphabetic()) {
            break;
        }
        *cursor += 1;
    }
    String::from_utf8(s[start..*cursor].to_vec()).unwrap()
}

fn look_ahead_is(s: &[u8], cursor: usize, expected: u8) -> bool {
    s.get(cursor).copied() == Some(expected)
}

pub fn tokenize(s: &[u8]) -> TokenContainer {
    let mut vec = Vec::new();
    let mut cursor = 0;
    while cursor < s.len() {
        match s[cursor] {
            c if c.is_ascii_whitespace() => {
                cursor += 1;
            }
            b'!' => {
                if look_ahead_is(s, cursor + 1, b'=') {
                    vec.push(Token {
                        kind: TokenKind::Ne,
                        span: Span {
                            pos: cursor,
                            len: 2,
                        },
                    });
                    cursor += 2;
                } else {
                    panic!("Not Supported Operator!")
                }
            }
            b'=' => {
                if look_ahead_is(s, cursor + 1, b'=') {
                    vec.push(Token {
                        kind: TokenKind::EqEq,
                        span: Span {
                            pos: cursor,
                            len: 2,
                        },
                    });
                    cursor += 2;
                } else {
                    vec.push(Token {
                        kind: TokenKind::Eq,
                        span: Span {
                            pos: cursor,
                            len: 1,
                        },
                    });
                    cursor += 1;
                }
            }
            b'>' => {
                if look_ahead_is(s, cursor + 1, b'=') {
                    vec.push(Token {
                        kind: TokenKind::Ge,
                        span: Span {
                            pos: cursor,
                            len: 2,
                        },
                    });
                    cursor += 2;
                } else {
                    vec.push(Token {
                        kind: TokenKind::Gt,
                        span: Span {
                            pos: cursor,
                            len: 1,
                        },
                    });
                    cursor += 1;
                }
            }
            b'<' => {
                if look_ahead_is(s, cursor + 1, b'=') {
                    vec.push(Token {
                        kind: TokenKind::Le,
                        span: Span {
                            pos: cursor,
                            len: 2,
                        },
                    });
                    cursor += 2;
                } else {
                    vec.push(Token {
                        kind: TokenKind::Lt,
                        span: Span {
                            pos: cursor,
                            len: 1,
                        },
                    });
                    cursor += 1;
                }
            }
            b'+' => {
                vec.push(Token {
                    kind: TokenKind::Add,
                    span: Span {
                        pos: cursor,
                        len: 1,
                    },
                });
                cursor += 1;
            }
            b'-' => {
                vec.push(Token {
                    kind: TokenKind::Sub,
                    span: Span {
                        pos: cursor,
                        len: 1,
                    },
                });
                cursor += 1;
            }
            b'*' => {
                vec.push(Token {
                    kind: TokenKind::Mul,
                    span: Span {
                        pos: cursor,
                        len: 1,
                    },
                });
                cursor += 1;
            }
            b'/' => {
                vec.push(Token {
                    kind: TokenKind::Div,
                    span: Span {
                        pos: cursor,
                        len: 1,
                    },
                });
                cursor += 1;
            }
            b'(' => {
                vec.push(Token {
                    kind: TokenKind::LParen,
                    span: Span {
                        pos: cursor,
                        len: 1,
                    },
                });
                cursor += 1;
            }
            b')' => {
                vec.push(Token {
                    kind: TokenKind::RParen,
                    span: Span {
                        pos: cursor,
                        len: 1,
                    },
                });
                cursor += 1;
            }
            b'{' => {
                vec.push(Token {
                    kind: TokenKind::LBrace,
                    span: Span {
                        pos: cursor,
                        len: 1,
                    },
                });
                cursor += 1;
            }
            b'}' => {
                vec.push(Token {
                    kind: TokenKind::RBrace,
                    span: Span {
                        pos: cursor,
                        len: 1,
                    },
                });
                cursor += 1;
            }
            b';' => {
                vec.push(Token {
                    kind: TokenKind::Semi,
                    span: Span {
                        pos: cursor,
                        len: 1,
                    },
                });
                cursor += 1;
            }
            b',' => {
                vec.push(Token {
                    kind: TokenKind::Comma,
                    span: Span {
                        pos: cursor,
                        len: 1,
                    },
                });
                cursor += 1;
            }
            ident if ident.is_ascii_alphabetic() => {
                let pos = cursor;
                let data = parse_next_ident(s, &mut cursor);
                let kind = if let Some(kw) = KeywordKind::lex_keyword(data.as_str()) {
                    TokenKind::Keyword(kw)
                } else {
                    TokenKind::Ident(data)
                };
                vec.push(Token {
                    kind,
                    span: Span {
                        pos,
                        len: cursor - pos,
                    },
                });
            }
            c if c.is_ascii_digit() => {
                let pos = cursor;
                vec.push(Token {
                    kind: TokenKind::Literal(Lit {
                        kind: LitKind::Integer,
                        symbol: parse_next_number(s, &mut cursor),
                    }),
                    span: Span {
                        pos,
                        len: cursor - pos,
                    },
                });
            }
            b'\n' => {
                cursor += 1;
            }
            _ => {
                panic!("unexpected token");
            }
        }
    }
    vec
}
