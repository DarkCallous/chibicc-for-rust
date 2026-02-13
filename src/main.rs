use chibicc_for_rust::codegen::abi::{sysv::*, win64::*};
use chibicc_for_rust::codegen::*;
use chibicc_for_rust::parser::*;
use chibicc_for_rust::span::*;
use chibicc_for_rust::{span::source_map::SourceFile, tokenizer::*};
use std::env::{self};

fn main() {
    let cmds: Vec<String> = env::args().collect();
    let input_str = cmds.get(1).expect("should have at least 1 input").as_str();
    let source_file = SourceFile::new(source_map::FileName::Cli, input_str.to_string());
    let tokens = tokenize(input_str.as_bytes());

    let mut parser = Parser {
        tokens,
        index: 0,
        errors: vec![],
        locals: vec![],
        expr_cnt: 0,
    };

    let ast = parser.parse_crate();
    if parser.errors.is_empty() {
        let _ = gen_asm::<Win64Abi>(ast, &parser.locals);
    }
    for e in parser.errors {
        e.error_print(&source_file);
    }
}
