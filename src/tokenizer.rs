use crate::span::*;

#[derive(Debug, Clone, PartialEq)]
pub enum LitKind{
    Integer,
    Str,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Lit{
    pub kind: LitKind,
    pub symbol: String,
}

#[derive(Debug, PartialEq, Clone)]
pub enum TokenKind{
    Literal(Lit),
    Add,
    Sub,
    Mul,
    Div,
    LParen,
    RParen,
    Reserved(String),
    Eof,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Token{
    pub kind: TokenKind,
    pub span: Span,
}

pub type TokenContainer = Vec<Token>;

pub fn parse_next_number(s: &[u8], cursor: &mut usize) -> String{
    let start = *cursor;
    while let Some(c) = s.get(*cursor){
        if !c.is_ascii_digit() {break;}
        *cursor += 1;
    }
    String::from_utf8(s[start..*cursor].to_vec()).unwrap()
}

pub fn tokenize(s: &[u8]) -> TokenContainer{
    let mut vec = Vec::new();
    let mut cursor = 0;
    while cursor < s.len(){
        match s[cursor]{
            c if c.is_ascii_whitespace() =>{
                cursor += 1;    
            }
            b'+'=>{
                vec.push(Token{
                            kind: TokenKind::Add, 
                            span: Span{pos: cursor, len: 1}});
                cursor += 1;
            }
            b'-'=>{
                vec.push(Token{
                            kind: TokenKind::Sub, 
                            span: Span{pos: cursor, len: 1}});
                cursor += 1;
            }
            b'*'=>{
                vec.push(Token{
                            kind: TokenKind::Mul, 
                            span: Span{pos: cursor, len: 1}});
                cursor += 1;
            }
            b'/'=>{
                vec.push(Token{
                            kind: TokenKind::Div, 
                            span: Span{pos: cursor, len: 1}});
                cursor += 1;
            }
            b'('=>{
                vec.push(Token{
                            kind: TokenKind::LParen, 
                            span: Span{pos: cursor, len: 1}});
                cursor += 1;
            }
            b')'=>{
                vec.push(Token{
                            kind: TokenKind::RParen, 
                            span: Span{pos: cursor, len: 1}});
                cursor += 1;
            }
            c if c.is_ascii_digit() =>{
                let pos = cursor;
                vec.push(Token{
                    kind: TokenKind::Literal(
                        Lit{kind: LitKind::Integer, symbol: parse_next_number(s, &mut cursor)}), 
                    span: Span{pos, len: cursor - pos}});
            }
            _ =>{
                panic!("unexpected token");
            }
        }
    }
    vec
}