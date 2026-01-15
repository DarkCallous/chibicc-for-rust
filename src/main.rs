use std::{env::{self}};
use chibicc_for_rust::tokenizer::*;
use chibicc_for_rust::parser::*;
use chibicc_for_rust::ast::*;

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
    gen_asm(ast);
    println!("  pop rax\n");
    println!("  ret\n");
    for e in parser.errors{
        e.error_print(input_str);
    }
}

fn gen_asm(ast: Expr){
    match ast.kind{
        ExprKind::Binary(ops, lhs, rhs) => {
            gen_asm(*lhs);
            gen_asm(*rhs);
            println!("  pop rdi\n");
            println!("  pop rax\n");
            match ops {
                BinaryOpKind::Add => {
                    println!("  add rax, rdi\n");
                }
                BinaryOpKind::Sub => {
                    println!("  sub rax, rdi\n");
                }
                BinaryOpKind::Mul =>{
                    println!("  imul rax, rdi\n");
                } 
                BinaryOpKind::Div =>{
                    println!("  cqo\n");
                    println!("  idiv rdi\n");
                } 
            }
        }
        ExprKind::Literal(text) =>{
            println!("  push {}\n", text.symbol);
            return;
        }
        ExprKind::Error => {return}
    };
    println!("  push rax\n");
}



