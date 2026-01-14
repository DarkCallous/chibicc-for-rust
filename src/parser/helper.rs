use crate::tokenizer::*;
use crate::error_handler::*;

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