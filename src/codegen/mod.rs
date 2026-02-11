use std::io::{self, Write};

use crate::{ast::*, codegen::context::ProgContext};

mod context;

pub struct CodeGen<W: Write> {
    writer: W,
}

impl<W: Write> Write for CodeGen<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.writer.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

impl<W: Write> CodeGen<W> {
    pub fn new(writer: W) -> CodeGen<W> {
        CodeGen { writer }
    }

    pub fn gen_expr(&mut self, expr: &Expr, locals: &[String]) -> Result<(), io::Error> {
        match &expr.kind {
            ExprKind::Binary(ops, lhs, rhs) => {
                self.gen_expr(rhs, locals)?;
                writeln!(self, "  push rax\n")?;
                self.gen_expr(lhs, locals)?;
                writeln!(self, "  pop rdi\n")?;
                match ops {
                    cmp @ (BinaryOpKind::EqEq
                    | BinaryOpKind::Ne
                    | BinaryOpKind::Ge
                    | BinaryOpKind::Gt
                    | BinaryOpKind::Le
                    | BinaryOpKind::Lt) => {
                        writeln!(self, "  cmp rax, rdi\n")?;
                        match cmp {
                            BinaryOpKind::EqEq => {
                                writeln!(self, "  sete al\n")?;
                            }
                            BinaryOpKind::Ne => {
                                writeln!(self, "  setne al\n")?;
                            }
                            BinaryOpKind::Ge => {
                                writeln!(self, "  setge al\n")?;
                            }
                            BinaryOpKind::Gt => {
                                writeln!(self, "  setg al\n")?;
                            }
                            BinaryOpKind::Le => {
                                writeln!(self, "  setle al\n")?;
                            }
                            BinaryOpKind::Lt => {
                                writeln!(self, "  setl al\n")?;
                            }
                            _ => unreachable!(),
                        }
                        writeln!(self, "  movzx rax, al\n")?;
                    }
                    BinaryOpKind::Add => {
                        writeln!(self, "  add rax, rdi\n")?;
                    }
                    BinaryOpKind::Sub => {
                        writeln!(self, "  sub rax, rdi\n")?;
                    }
                    BinaryOpKind::Mul => {
                        writeln!(self, "  imul rax, rdi\n")?;
                    }
                    BinaryOpKind::Div => {
                        writeln!(self, "  cqo\n")?;
                        writeln!(self, "  idiv rdi\n")?;
                    }
                }
            }
            ExprKind::Unary(op, operand) => {
                self.gen_expr(operand, locals)?; // Generate code for operand (pushes result)

                match op {
                    UnaryOpKind::Pos => {
                        // Unary '+' does nothing - value already on stack
                        // No code needed!
                    }
                    UnaryOpKind::Neg => {
                        // Negate the value on top of stack
                        writeln!(self, "  neg rax\n")?; // Negate it (rax = -rax)
                    }
                }
            }
            ExprKind::Literal(text) => {
                writeln!(self, "  mov rax, {}\n", text.symbol)?;
            }
            ExprKind::Var(_) => {
                self.gen_var(expr, locals)?;
                writeln!(self, "  mov rax, [rax]\n")?;
            }
            ExprKind::Assign(lhs, rhs) => {
                self.gen_var(lhs, locals)?;
                writeln!(self, "  push rax\n")?;
                self.gen_expr(rhs, locals)?;
                writeln!(self, "  pop rdi\n")?;
                writeln!(self, "  mov [rdi], rax\n")?;
            }
            ExprKind::FnCall(sym) => {
                writeln!(self, "  call {sym}\n")?;
            }
            ExprKind::Error => {}
        };
        Ok(())
    }

    pub fn gen_var(&mut self, var: &Expr, locals: &[String]) -> Result<(), io::Error> {
        if let ExprKind::Var(sym) = &var.kind {
            let offset = 8 * (locals.iter().position(|s| s == sym).unwrap() + 1);
            writeln!(self, "  lea rax, [rbp - {offset}]\n")?;
        } else {
            unreachable!("should not call gen_var on non-LValue");
        }
        Ok(())
    }

    pub fn gen_stmt(
        &mut self,
        stmt: &Stmt,
        locals: &[String],
        prog_context: &mut ProgContext,
    ) -> Result<(), io::Error> {
        match &stmt {
            Stmt::Block(stmts) => {
                for stmt in stmts {
                    self.gen_stmt(stmt, locals, prog_context)?;
                }
            }
            Stmt::ExprStmt(expr) => {
                self.gen_expr(expr, locals)?;
            }
            Stmt::Return(expr) => {
                self.gen_expr(expr, locals)?;
                writeln!(self, "  jmp .L.return\n")?;
            }
            Stmt::If(condition, ops, else_ops) => {
                self.gen_expr(condition, locals)?;
                let cnt = prog_context.apply();
                writeln!(self, "  cmp rax, 0\n")?;
                writeln!(self, "  je .L.else.{}\n", cnt)?;
                self.gen_stmt(ops, locals, prog_context)?;
                writeln!(self, "  jmp .L.end.{}\n", cnt)?;
                writeln!(self, ".L.else.{}:\n", cnt)?;
                if let Some(else_ops) = &**else_ops {
                    self.gen_stmt(else_ops, locals, prog_context)?;
                }
                writeln!(self, ".L.end.{}:\n", cnt)?;
            }
            Stmt::For(init, cond, incr, ops) => {
                let cnt = prog_context.apply();
                if let Some(expr) = &**init {
                    self.gen_expr(expr, locals)?;
                }
                writeln!(self, ".L.begin.{}:\n", cnt)?;
                if let Some(expr) = &**cond {
                    self.gen_expr(expr, locals)?;
                    writeln!(self, "  cmp rax, 0\n")?;
                    writeln!(self, "  je  .L.end.{}\n", cnt)?;
                }
                self.gen_stmt(ops, locals, prog_context)?;
                if let Some(expr) = &**incr {
                    self.gen_expr(expr, locals)?;
                }
                writeln!(self, "  jmp .L.begin.{}\n", cnt)?;
                writeln!(self, ".L.end.{}:\n", cnt)?;
            }
            Stmt::While(cond, ops) => {
                let cnt = prog_context.apply();
                writeln!(self, ".L.begin.{}:\n", cnt)?;
                self.gen_expr(cond, locals)?;
                writeln!(self, "  cmp rax, 0\n")?;
                writeln!(self, "  je  .L.end.{}\n", cnt)?;
                self.gen_stmt(ops, locals, prog_context)?;
                writeln!(self, "  jmp .L.begin.{}\n", cnt)?;
                writeln!(self, ".L.end.{}:\n", cnt)?;
            }
            Stmt::Null => {}
        }
        Ok(())
    }
}

pub fn gen_asm(crat: Crate, locals: &[String]) -> Result<(), io::Error> {
    let mut crate_context = ProgContext::new();
    println!(".intel_syntax noprefix\n");
    println!(".globl main\n");
    println!("main:\n");

    println!("  push rbp\n");
    println!("  mov rbp, rsp\n");
    println!("  sub rsp, {}\n", locals.len() * 8);

    let mut codegen = CodeGen::new(io::stdout());
    for exp in crat.stmts {
        codegen.gen_stmt(&exp, locals, &mut crate_context)?;
    }
    println!(".L.return:\n");
    println!("  mov rsp, rbp\n");
    println!("  pop rbp\n");
    println!("  ret\n");
    Ok(())
}
