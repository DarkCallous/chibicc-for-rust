use abi::{Abi, Reg};
use std::io::{self, Write};

use crate::{ast::*, codegen::context::{FnContext, ProgContext}};

mod context;

pub mod abi;

pub struct CodeGen<W: Write, ABI: Abi + Default> {
    writer: W,
    abi: ABI,
}

impl<W: Write, ABI: Abi + Default> Write for CodeGen<W, ABI> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.writer.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

impl<W: Write, ABI: Abi + Default> CodeGen<W, ABI> {
    pub fn new(writer: W) -> CodeGen<W, ABI> {
        CodeGen {
            writer,
            abi: ABI::default(),
        }
    }

    pub fn push(&mut self, reg: &Reg) -> Result<(), io::Error> {
        writeln!(self, "  push {}\n", reg.asm())
    }

    pub fn pop(&mut self, reg: &Reg) -> Result<(), io::Error> {
        writeln!(self, "  pop {}\n", reg.asm())
    }

    pub fn gen_expr(&mut self, expr: &Expr, locals: &[String]) -> Result<(), io::Error> {
        match &expr.kind {
            ExprKind::Binary(ops, lhs, rhs) => {
                self.gen_expr(rhs, locals)?;
                self.push(&Reg::Rax)?;
                self.gen_expr(lhs, locals)?;
                self.pop(&Reg::Rdi)?;
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
                self.push(&Reg::Rax)?;
                self.gen_expr(rhs, locals)?;
                self.pop(&Reg::Rdi)?;
                writeln!(self, "  mov [rdi], rax\n")?;
            }
            ExprKind::FnCall(sym, exprs) => {
                for expr in exprs.iter().rev() {
                    self.gen_expr(expr, locals)?;
                    self.push(&Reg::Rax)?;
                }
                let regs = self.abi.int_arg_regs();
                let nreg = exprs.len().min(regs.len());
                for reg in self.abi.int_arg_regs().iter().take(nreg) {
                    self.pop(reg)?;
                }
                let shadow = self.abi.shadow_space_size();
                if shadow > 0 {
                    writeln!(self, "  sub rsp, {shadow}\n")?;
                }
                writeln!(self, "  call {sym}\n")?;
                let collect = shadow + exprs.len().saturating_sub(nreg) * 8;
                if collect > 0 {
                    writeln!(self, "  add rsp, {collect}\n")?;
                }
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

    pub fn gen_fn(&mut self, crat: Crate, locals: &[String], mut crate_context: ProgContext, name: String)->Result<(), io::Error> {
        let context = FnContext::new(name);
        println!("{}:\n", context.name);
        writeln!(self, "  push rbp\n")?;
        writeln!(self, "  mov rbp, rsp\n")?;
        writeln!(self, "  sub rsp, {}\n", locals.len() * 8)?;
        for exp in crat.stmts {
            self.gen_stmt(&exp, locals, &mut crate_context)?;
        }
        writeln!(self, ".L.return:\n")?;
        writeln!(self, "  mov rsp, rbp\n")?;
        writeln!(self, "  pop rbp\n")?;
        writeln!(self, "  ret\n")?;
        Ok(())
    }
}

pub fn gen_asm<ABI: Abi + Default>(crat: Crate, locals: &[String]) -> Result<(), io::Error> {
    let crate_context = ProgContext::new();
    println!(".intel_syntax noprefix\n");
    println!(".globl main\n");
    let mut codegen: CodeGen<io::Stdout, ABI> = CodeGen::new(io::stdout());
    codegen.gen_fn(crat, locals, crate_context, "main".to_string())?;
    Ok(())
}
