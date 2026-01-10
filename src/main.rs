use std::env::{self};

fn main() {
    let cmds: Vec<String> = env::args().collect();
    let input = cmds.get(1).expect("should have at least one input");
    println!(".intel_syntax noprefix\n");
    println!(".globl main\n");
    println!("main:\n");
    println!("  mov rax, {}\n", input);
    println!("  ret\n");
}
