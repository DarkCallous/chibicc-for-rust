#[derive(Debug)]
pub enum NextTokenError<'a>{
    WrongType(&'a Token),
}

#[derive(Debug)]
pub enum Token{
    Num(i64),
    Reserved(String),
    Eof,
}

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
                vec.push(Token::Reserved((s[cursor] as char).to_string()));
                cursor += 1;
            }
            c if c.is_ascii_digit() =>{
                vec.push(Token::Num(parse_next_number(s, &mut cursor)));
            }
            _ =>{
                panic!("unexpected token");
            }
        }
    }
    vec
}

pub fn next_num<'a>(tokens: &'a [Token], index: &mut usize) -> Result<i64, NextTokenError<'a>>{
    let result = match &tokens[*index]{
        Token::Num(i) => {
            Ok(*i)
        }
        c => {
            Err(NextTokenError::WrongType(c))
        }
    };
    *index += 1;
    result
}

pub fn next_reserve<'a>(tokens: &'a [Token], index: &mut usize) -> Result<&'a str, NextTokenError<'a>>{
    let result: Result<&'a str, NextTokenError<'a>> = match &tokens[*index]{
        Token::Reserved(s) => {
            Ok(&s)
        }
        c => {
            Err(NextTokenError::WrongType(c))
        }
    };
    *index += 1;
    result
}