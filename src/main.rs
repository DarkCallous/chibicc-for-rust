use std::{env::{self}};
use chibicc_for_rust::tokenizer::*;
use chibicc_for_rust::parser::*;

fn main() {
    let cmds: Vec<String> = env::args().collect();
    let input_str = cmds.get(1).expect("should have at least 1 input").as_str();
    let tokens = tokenize(input_str.as_bytes());
    println!(".intel_syntax noprefix\n");
    println!(".globl main\n");
    println!("main:\n");
    let mut parser = Parser{
        tokens,
        index: 0,
        errors: vec![],
    };
    let ast = parser.parse_expr();
    for e in parser.errors{
        e.error_print(input_str);
    }
    println!("ast gen");
}



