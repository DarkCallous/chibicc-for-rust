use crate::span::*;

static STR_NUMBER: &str = "number";
static STR_RESERVE: &str = "reserve";

#[derive(Debug)]
pub enum NextTokenError<'a>{
    WrongType{
        expected: &'static str,
        found: &'a Token,
    },
}

impl<'a> NextTokenError<'a>{
    pub fn error_print(&'a self, source: &'a str){
         match self {
            NextTokenError::WrongType { expected, found } => {
                let line_number = 1;
                let col_number = found.pos + 1; // pos 从 0 开始，所以 +1

                // 打印行内容
                println!("Error: expected {}, found {:?} at line {}, column {}",
                         expected, found.item, line_number, col_number);
                println!("{}", source);

                // 打印箭头指示位置
                let mut marker = String::new();
                for _ in 0..found.pos {
                    marker.push(' ');
                }
                for _ in 0..found.len {
                    marker.push('^');
                }
                println!("{}", marker);
            }
        }
    }
}


#[derive(Debug)]
pub enum TokenKind{
    Num(i64),
    Reserved(String),
    Eof,
}

type Token = Span<TokenKind>;

type TokenContainer = Vec<Token>;

fn parse_next_number(s: &[u8], cursor: &mut usize) -> i64{
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

pub fn next_num<'a>(tokens: &'a [Token], index: &mut usize) -> Result<i64, NextTokenError<'a>>{
    let token = &tokens[*index];
    let result = match &token.item{
        TokenKind::Num(i) => {
            Ok(*i)
        }
        _ => {
            Err(NextTokenError::WrongType{expected: STR_NUMBER, found: token})
        }
    };
    *index += 1;
    result
}

pub fn next_reserve<'a>(tokens: &'a [Token], index: &mut usize) -> Result<&'a str, NextTokenError<'a>>{
    let token = &tokens[*index];
    let result = match &token.item{
        TokenKind::Reserved(s) => {
            Ok(s.as_str())
        }
        _ => {
            Err(NextTokenError::WrongType{expected: STR_RESERVE, found: token})
        }
    };
    *index += 1;
    result
}
