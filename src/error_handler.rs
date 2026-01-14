pub use crate::tokenizer::*;

pub static STR_NUMBER: &str = "number";
pub static STR_RESERVE: &str = "reserve";

#[derive(Debug)]
pub enum NextTokenError{
    WrongType{
        expected: &'static str,
        found: Token,
    },
}

impl NextTokenError{
    pub fn error_print(&self, source: &str){
         match self {
            NextTokenError::WrongType { expected, found } => {
                let line_number = 1;
                let col_number = found.span.pos + 1; // pos 从 0 开始，所以 +1

                // 打印行内容
                println!("Error: expected {}, found {:?} at line {}, column {}",
                         expected, found, line_number, col_number);
                println!("{}", source);

                // 打印箭头指示位置
                let mut marker = String::new();
                for _ in 0..found.span.pos {
                    marker.push(' ');
                }
                for _ in 0..found.span.len {
                    marker.push('^');
                }
                println!("{}", marker);
            }
        }
    }
}
