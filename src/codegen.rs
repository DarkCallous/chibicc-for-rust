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
                    cmp @ (BinaryOpKind::EqEq | BinaryOpKind::Ne | BinaryOpKind::Ge | 
           BinaryOpKind::Gt | BinaryOpKind::Le | BinaryOpKind::Lt) =>{
                        println!("  cmp rax, rdi\n");
                        match cmp{
                            BinaryOpKind::EqEq =>{
                                println!("  sete al\n");
                            }
                            BinaryOpKind::Ne =>{
                                println!("  setne al\n");
                            }
                            BinaryOpKind::Ge =>{
                                println!("  setge al\n");
                            }
                            BinaryOpKind::Gt =>{
                                println!("  setg al\n");
                            }
                            BinaryOpKind::Le =>{
                                println!("  setle al\n");
                            }
                            BinaryOpKind::Lt =>{
                                println!("  setl al\n");
                            }
                            _ => unreachable!()
                        }
                        println!("  movzx rax, al\n");
                    }
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
            ExprKind::Unary(op, operand) => {
                operand.gen_asm();  // Generate code for operand (pushes result)
                
                match op {
                    UnaryOpKind::Pos => {
                        // Unary '+' does nothing - value already on stack
                        // No code needed!
                    }
                    UnaryOpKind::Neg => {
                        // Negate the value on top of stack
                        println!("  pop rax");       // Get value from stack
                        println!("  neg rax");       // Negate it (rax = -rax)
                        println!("  push rax");      // Push result back
                    }
                }
                return;
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
