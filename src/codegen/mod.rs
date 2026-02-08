use crate::{ast::*, codegen::context::ProgContext};

mod context;

pub fn gen_asm(crat: Crate, locals: &[String]){
    let mut crate_context = ProgContext::new();
    println!(".intel_syntax noprefix\n");
    println!(".globl main\n");
    println!("main:\n");
    
    println!("  push rbp\n");
    println!("  mov rbp, rsp\n");
    println!("  sub rsp, {}\n", locals.len() * 8);

    for exp in crat.stmts{
        exp.gen_asm(locals, &mut crate_context);
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
                println!("  push rax\n");
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
                        println!("  pop rax\n");       // Get value from stack
                        println!("  neg rax\n");       // Negate it (rax = -rax)
                        println!("  push rax\n");      // Push result back
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
    fn gen_asm(&self, locals: &[String], prog_context: &mut ProgContext){
        match &self {
            Stmt::Block(stmts)=>{
                for stmt in stmts{
                    stmt.gen_asm(locals, prog_context);
                }
            }
            Stmt::ExprStmt(expr)=>{
                expr.gen_asm(locals);
                println!("  pop rax\n");
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
                let cnt = prog_context.apply();
                println!("  cmp rax, 0\n");
                println!("  je .L.else.{}\n", cnt);
                ops.gen_asm(locals, prog_context);
                println!("  jmp .L.end.{}\n", cnt);
                println!(".L.else.{}:\n", cnt);
                if let Some(else_ops) = &**else_ops{
                    else_ops.gen_asm(locals, prog_context);
                }
                println!(".L.end.{}:\n", cnt);
                return;
            }
            Stmt::For(init, cond, incr, ops)=>{
                let cnt = prog_context.apply();
                if let Some(expr) = &**init{
                    expr.gen_asm(locals);
                }
                println!(".L.begin.{}:\n", cnt);
                if let Some(expr) = &**cond{
                    expr.gen_asm(locals);
                    println!("  cmp rax, 0\n");
                    println!("  je  .L.end.{}\n", cnt);
                }
                ops.gen_asm(locals, prog_context);
                if let Some(expr) = &**incr{
                    expr.gen_asm(locals);
                }
                println!("  jmp .L.begin.{}\n", cnt);
                println!(".L.end.{}:\n", cnt);
                return;
            }
            Stmt::While(cond, ops)=>{
                let cnt = prog_context.apply();
                println!(".L.begin.{}:\n", cnt);     
                cond.gen_asm(locals);
                println!("  cmp rax, 0\n");
                println!("  je  .L.end.{}\n", cnt);
                ops.gen_asm(locals, prog_context);
                println!("  jmp .L.begin.{}\n", cnt);
                println!(".L.end.{}:\n", cnt);
                return;
            }
            Stmt::Null=>{}
            _=>todo!()
        }
    }
}