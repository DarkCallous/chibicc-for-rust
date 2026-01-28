use crate::ast::*;

mod context;

pub fn gen_asm(crat: Crate, locals: &[String]){
    println!(".intel_syntax noprefix\n");
    println!(".globl main\n");
    println!("main:\n");
    
    println!("  push rbp\n");
    println!("  mov rbp, rsp\n");
    println!("  sub rsp, {}\n", locals.len() * 8);

    for exp in crat.stmts{
        exp.gen_asm(locals);
    }
    println!("  mov rsp, rbp\n");
    println!("  pop rbp\n");
    println!("  ret\n");
}

pub fn gen_var(var: &Expr, locals: &[String]){
    if let ExprKind::Var(sym) = &var.kind{
        println!("  mov rax, rbp\n");
        let offset = 8*(locals.iter().position(|s| s == sym).unwrap() + 1);
        println!("  sub rax, {}\n", offset);
        println!("  push rax\n");
    }
    else {
        unreachable!("should not call gen_var on non-LValue");
    }
}

impl Expr{
    fn gen_asm(&self, locals: &[String]){
        match &self.kind{
            ExprKind::Binary(ops, lhs, rhs) => {
                lhs.gen_asm(locals);
                rhs.gen_asm(locals);
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
                println!("  push rax");
            }
            ExprKind::Unary(op, operand) => {
                operand.gen_asm(locals);  // Generate code for operand (pushes result)
                
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
            ExprKind::Var(_) =>{
                gen_var(&self, locals);
                println!("  pop rax\n");
                println!("  mov rax, [rax]\n");
                println!("  push rax\n");
                return;
            }
            ExprKind::Assign(lhs, rhs) => {
                gen_var(lhs,locals);
                rhs.as_ref().gen_asm(locals);
                println!("  pop rdi\n");
                println!("  pop rax\n");
                println!("  mov [rax], rdi\n");
                println!("  push rdi\n");
                return;
            }
            ExprKind::Error => {return}
        };
    }
}

impl Stmt{
    fn gen_asm(&self, locals: &[String]){
        match &self {
            Stmt::ExprStmt(expr)=>{
                expr.gen_asm(locals);
            }
            Stmt::Return(expr)=>{
                expr.gen_asm(locals);
                println!("  pop rax\n");
                println!("  mov rsp, rbp\n");
                println!("  pop rbp\n");
                println!("  ret\n");
                return
            }
            Stmt::If(condition, ops, else_ops)=>{
                condition.gen_asm(locals);
                let cnt = 0;
                println!("  cmp $0, %%rax\n");
                println!("  je .L.else.{}\n", cnt);
                ops.gen_asm(locals);
                println!("  jmp .L.end.{}\n", cnt);
                println!(".L.else.{}:\n", cnt);
                if let Some(else_ops) = &**else_ops{
                    else_ops.gen_asm(locals);
                }
                println!(".L.end.{}:\n", cnt);
                return;
            }
            Stmt::Null=>{}
            _=>todo!()
        }
    }
}