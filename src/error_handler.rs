pub use crate::ast::*;
use crate::span::source_map::SourceFile;
pub use crate::tokenizer::*;

pub static STR_NUMBER: &str = "number";
pub static STR_RESERVE: &str = "reserve";

#[derive(Debug, Clone)]
pub enum NextTokenError {
    WrongType {
        expected: &'static str,
        found: Token,
    },
    ExpectedToken {
        expected: TokenKind,
        found: Token,
    },
}

impl NextTokenError {
    pub fn error_print(&self, source: &SourceFile) {
        match self {
            NextTokenError::WrongType { expected, found } => {
                let (line, col) = source.lookup_line_column(found.span.pos);
                // 打印行内容
                println!(
                    "Error: expected {}, found {:?} at line {}, column {}",
                    expected, found, line, col
                );
                println!("{}", source.line_content(line));

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
            NextTokenError::ExpectedToken { expected, found } => {
                let (line, col) = source.lookup_line_column(found.span.pos);
                // 打印行内容
                println!(
                    "Error: expected {:?}, found {:?} at line {}, column {}",
                    expected, found, line, col
                );
                println!("{}", source.line_content(line));

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

    pub fn gen_error_expr(&self, id: &mut usize) -> Expr {
        let span = match &self {
            NextTokenError::WrongType { found, .. } => found.span,
            NextTokenError::ExpectedToken { found, .. } => found.span,
        };
        let result = Expr {
            id: *id, 
            kind: ExprKind::Error,
            span,
        };
        *id += 1;
        result
    }
}
