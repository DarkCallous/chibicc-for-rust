use std::{env::{self}, process};
use chibicc_for_rust::tokenizer::*;
use chibicc_for_rust::parser::helper::*;

fn main() {
    let cmds: Vec<String> = env::args().collect();
    let input_str = cmds.get(1).expect("should have at least 1 input").as_str();
    let tokens = tokenize(input_str.as_bytes());
    println!(".intel_syntax noprefix\n");
    println!(".globl main\n");
    println!("main:\n");
    let mut index = 0;
    let num = next_num(&tokens, &mut index)
        .unwrap_or_else(|err| {err.error_print(input_str); process::exit(1)});
    println!("  mov rax, {}\n", num);
    while index < tokens.len()  {
        let char = next_reserve(&tokens, &mut index)
            .unwrap_or_else(|err| {err.error_print(input_str); process::exit(1)});
        match char{
            "+" => {
                let num = next_num(&tokens, &mut index)
                    .unwrap_or_else(|err| {err.error_print(input_str); process::exit(1)});
                println!("  add rax, {}\n", num);
            }
            "-" => {
                let num = next_num(&tokens, &mut index)
                    .unwrap_or_else(|err| {err.error_print(input_str); process::exit(1)});
                println!("  sub rax, {}\n", num);
            }
            _ => {
                panic!("found not expected char!");
            }
        }
    }
    println!("  ret\n");
}



