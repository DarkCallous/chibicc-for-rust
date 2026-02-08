use std::{env::{self}};
use chibicc_for_rust::{span::source_map::SourceFile, tokenizer::*};
use chibicc_for_rust::parser::*;
use chibicc_for_rust::codegen::*;
use chibicc_for_rust::span::*;

fn main() {
    let cmds: Vec<String> = env::args().collect();
    let input_str = cmds.get(1).expect("should have at least 1 input").as_str();
    let source_file = SourceFile::new(source_map::FileName::Cli, input_str.to_string());
    let tokens = tokenize(input_str.as_bytes());
    
    let mut parser = Parser{
        tokens,
        index: 0,
        errors: vec![],
        locals: vec![],
    };
    let ast = parser.parse_crate();
    if parser.errors.is_empty(){
        gen_asm(ast, &parser.locals);
    }
    for e in parser.errors{
        e.error_print(&source_file);
    }
}





