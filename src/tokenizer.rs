use crate::span::*;

#[derive(Debug)]
pub enum TokenKind{
    Num(i64),
    Reserved(String),
    Eof,
}

pub type Token = Span<TokenKind>;

pub type TokenContainer = Vec<Token>;

pub fn parse_next_number(s: &[u8], cursor: &mut usize) -> i64{
    let mut val: i64 = 0;
    while let Some(c) = s.get(*cursor){
        if !c.is_ascii_digit() {break;}
        val = val * 10 + (c - b'0') as i64;
        *cursor += 1;
    }
    val
}

pub fn tokenize(s: &[u8]) -> TokenContainer{
    let mut vec = Vec::new();
    let mut cursor = 0;
    while cursor < s.len(){
        match s[cursor]{
            c if c.is_ascii_whitespace() =>{
                cursor += 1;    
            }
            b'+' | b'-' =>{
                vec.push(Token{item: TokenKind::Reserved((s[cursor] as char).to_string()), pos: cursor, len: 1});
                cursor += 1;
            }
            c if c.is_ascii_digit() =>{
                let pos = cursor;
                vec.push(Token{item: TokenKind::Num(parse_next_number(s, &mut cursor)), pos, len: cursor - pos});
            }
            _ =>{
                panic!("unexpected token");
            }
        }
    }
    vec
}