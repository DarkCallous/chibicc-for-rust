use crate::ast::*;

pub fn gen_asm(exp: Expr){
    println!(".intel_syntax noprefix\n");
    println!(".globl main\n");
    println!("main:\n");
    exp.gen_asm();
    println!("  pop rax\n");
    println!("  ret\n");
}


impl Expr{
    fn gen_asm(self){
        match self.kind{
            ExprKind::Binary(ops, lhs, rhs) => {
                lhs.gen_asm();
                rhs.gen_asm();
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
}
