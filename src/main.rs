use std::env::{self};
use chibicc_for_rust::tokenizer::*;


fn main() {
    let cmds: Vec<String> = env::args().collect();
    let input_str = cmds.get(1).expect("should have at least 1 input").as_str();
    let tokens = tokenize(input_str.as_bytes());
    println!(".intel_syntax noprefix\n");
    println!(".globl main\n");
    println!("main:\n");
    let mut index = 0;
    let num = next_num(&tokens, &mut index).expect("first token must be number");
    println!("  mov rax, {}\n", num);
    while index < tokens.len()  {
        let char = next_reserve(&tokens, &mut index).expect("next token must be operator");
        match char{
            "+" => {
                let num = next_num(&tokens, &mut index).expect("next token must be number");
                println!("  add rax, {}\n", num);
            }
            "-" => {
                let num = next_num(&tokens, &mut index).expect("next token must be number");
                println!("  sub rax, {}\n", num);
            }
            _ => {
                panic!("found not expected char!");
            }
        }
    }
    println!("  ret\n");
}

