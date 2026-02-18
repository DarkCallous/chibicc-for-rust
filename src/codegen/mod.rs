use super::resolver::*;
use abi::{Abi, Reg};
use std::io::{self, Write};

use crate::{
    ast::*,
    codegen::context::FnContext,
    frame_layout::{FrameLayout, FrameLayouts},
    resolver::ResolvedCrate,
};

mod context;

pub mod abi;

pub struct CodeGen<W: Write, ABI: Abi + Default> {
    writer: W,
    abi: ABI,
    resolved: ResolvedCrate,
    layouts: FrameLayouts,
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
    pub fn new(writer: W, resolved: ResolvedCrate, layouts: FrameLayouts) -> CodeGen<W, ABI> {
        CodeGen {
            writer,
            abi: ABI::default(),
            resolved,
            layouts,
        }
    }

    pub fn push(&mut self, reg: &Reg) -> Result<(), io::Error> {
        writeln!(self, "  push {}\n", reg.asm())
    }

    pub fn pop(&mut self, reg: &Reg) -> Result<(), io::Error> {
        writeln!(self, "  pop {}\n", reg.asm())
    }

    pub fn gen_expr(&mut self, expr: &Expr, fn_layout: &FrameLayout) -> Result<(), io::Error> {
        match &expr.kind {
            ExprKind::Binary(ops, lhs, rhs) => {
                self.gen_expr(rhs, fn_layout)?;
                self.push(&Reg::Rax)?;
                self.gen_expr(lhs, fn_layout)?;
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
                self.gen_expr(operand, fn_layout)?; // Generate code for operand (pushes result)

                match op {
                    UnaryOpKind::Pos => {
                        // Unary '+' does nothing - value already on stack
                        // No code needed!
                    }
                    UnaryOpKind::Neg => {
                        // Negate the value on top of stack
                        writeln!(self, "  neg rax\n")?; // Negate it (rax = -rax)
                    }
                    UnaryOpKind::AddrOf => {
                        self.gen_var(operand, fn_layout)?;
                    }
                    UnaryOpKind::Deref => {
                        writeln!(self, "  mov rax, [rax]")?;
                    }
                }
            }
            ExprKind::Literal(text) => {
                writeln!(self, "  mov rax, {}\n", text.symbol)?;
            }
            ExprKind::Var(_) => {
                self.gen_var(expr, fn_layout)?;
                writeln!(self, "  mov rax, [rax]\n")?;
            }
            ExprKind::Assign(lhs, rhs) => {
                self.gen_var(lhs, fn_layout)?;
                self.push(&Reg::Rax)?;
                self.gen_expr(rhs, fn_layout)?;
                self.pop(&Reg::Rdi)?;
                writeln!(self, "  mov [rdi], rax\n")?;
            }
            ExprKind::FnCall(sym, exprs) => {
                for expr in exprs.iter().rev() {
                    self.gen_expr(expr, fn_layout)?;
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

    pub fn gen_var(&mut self, var: &Expr, fn_layout: &FrameLayout) -> Result<(), io::Error> {
        match &var.kind {
            ExprKind::Var(_) => {
                let obj_id = self.resolved.expr_resolutions[&var.id];
                let offset = fn_layout.slots[&obj_id];
                writeln!(self, "  lea rax, [rbp - {offset}]\n")?;
            }
            ExprKind::Unary(op, inner) => {
                if op == &UnaryOpKind::Deref {
                    self.gen_expr(inner, fn_layout)?;
                } else {
                    unreachable!("should not call gen_var on non-LValue");
                }
            }
            _ => {
                unreachable!("should not call gen_var on non-LValue");
            }
        }
        Ok(())
    }

    pub fn gen_stmt(
        &mut self,
        stmt: &Stmt,
        prog_context: &mut FnContext,
        fn_info: &FnInfo,
        fn_layout: &FrameLayout,
    ) -> Result<(), io::Error> {
        match &stmt {
            Stmt::Block(stmts) => {
                for stmt in stmts {
                    self.gen_stmt(stmt, prog_context, fn_info, fn_layout)?;
                }
            }
            Stmt::ExprStmt(expr) => {
                self.gen_expr(expr, fn_layout)?;
            }
            Stmt::Return(expr) => {
                self.gen_expr(expr, fn_layout)?;
                writeln!(self, "  jmp .L.{}.return\n", fn_info.fn_id)?;
            }
            Stmt::If(condition, ops, else_ops) => {
                self.gen_expr(condition, fn_layout)?;
                let cnt = prog_context.apply();
                writeln!(self, "  cmp rax, 0\n")?;
                writeln!(self, "  je .L.{}.else.{}\n", fn_info.fn_id, cnt)?;
                self.gen_stmt(ops, prog_context, fn_info, fn_layout)?;
                writeln!(self, "  jmp .L.{}.end.{}\n", fn_info.fn_id, cnt)?;
                writeln!(self, ".L.{}.else.{}:\n", fn_info.fn_id, cnt)?;
                if let Some(else_ops) = &**else_ops {
                    self.gen_stmt(else_ops, prog_context, fn_info, fn_layout)?;
                }
                writeln!(self, ".L.{}.end.{}:\n", fn_info.fn_id, cnt)?;
            }
            Stmt::For(init, cond, incr, ops) => {
                let cnt = prog_context.apply();
                if let Some(expr) = &**init {
                    self.gen_expr(expr, fn_layout)?;
                }
                writeln!(self, ".L.{}.begin.{}:\n", fn_info.fn_id, cnt)?;
                if let Some(expr) = &**cond {
                    self.gen_expr(expr, fn_layout)?;
                    writeln!(self, "  cmp rax, 0\n")?;
                    writeln!(self, "  je  .L.{}.end.{}\n", fn_info.fn_id, cnt)?;
                }
                self.gen_stmt(ops, prog_context, fn_info, fn_layout)?;
                if let Some(expr) = &**incr {
                    self.gen_expr(expr, fn_layout)?;
                }
                writeln!(self, "  jmp .L.{}.begin.{}\n", fn_info.fn_id, cnt)?;
                writeln!(self, ".L.{}.end.{}:\n", fn_info.fn_id, cnt)?;
            }
            Stmt::While(cond, ops) => {
                let cnt = prog_context.apply();
                writeln!(self, ".L.{}.begin.{}:\n", fn_info.fn_id, cnt)?;
                self.gen_expr(cond, fn_layout)?;
                writeln!(self, "  cmp rax, 0\n")?;
                writeln!(self, "  je  .L.{}.end.{}\n", fn_info.fn_id, cnt)?;
                self.gen_stmt(ops, prog_context, fn_info, fn_layout)?;
                writeln!(self, "  jmp .L.{}.begin.{}\n", fn_info.fn_id, cnt)?;
                writeln!(self, ".L.{}.end.{}:\n", fn_info.fn_id, cnt)?;
            }
            Stmt::Null => {}
        }
        Ok(())
    }

    pub fn gen_fn(&mut self, func: Fn) -> Result<(), io::Error> {
        let mut context = FnContext::new(func.name);
        let fn_info = self.resolved.fn_info.remove(&context.name).unwrap();
        let fn_layout = self.layouts.fns.remove(&fn_info.fn_id).unwrap();
        writeln!(self, "{}:\n", context.name)?;
        writeln!(self, "  push rbp\n")?;
        writeln!(self, "  mov rbp, rsp\n")?;
        writeln!(self, "  sub rsp, {}\n", fn_layout.frame_size)?;

        let pair = self.abi.int_arg_regs().iter().zip(fn_info.params.iter());
        for (reg, param) in pair {
            writeln!(
                self,
                "  mov [rbp - {}], {}\n",
                fn_layout.slots[param],
                reg.asm()
            )?;
        }
        for param in self.abi.int_arg_regs().len()..fn_info.params.len() {
            let dest_offset = fn_layout.slots[&fn_info.params[param]];
            let src_offset = 8 * param.saturating_sub(self.abi.int_arg_regs().len())
                + self.abi.stack_param_base();
            writeln!(self, "  mov rax, [rbp + {}]", src_offset)?;
            writeln!(self, "  mov [rbp - {}], rax", dest_offset)?;
        }

        for exp in func.stmts {
            self.gen_stmt(&exp, &mut context, &fn_info, &fn_layout)?;
        }
        writeln!(self, ".L.{}.return:\n", fn_info.fn_id)?;
        writeln!(self, "  mov rsp, rbp\n")?;
        writeln!(self, "  pop rbp\n")?;
        writeln!(self, "  ret\n")?;
        Ok(())
    }

    pub fn gen_crate(&mut self, crat: Crate) -> Result<(), io::Error> {
        writeln!(self, ".intel_syntax noprefix\n")?;
        writeln!(self, ".globl main\n")?;
        for func in crat.fns {
            self.gen_fn(func)?;
        }
        Ok(())
    }
}

pub fn gen_asm<ABI: Abi + Default>(
    crat: Crate,
    res: ResolvedCrate,
    layouts: FrameLayouts,
) -> Result<(), io::Error> {
    let mut codegen: CodeGen<io::Stdout, ABI> = CodeGen::new(io::stdout(), res, layouts);
    codegen.gen_crate(crat)
}
