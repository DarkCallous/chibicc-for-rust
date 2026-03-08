use super::ast::{UnaryOpKind, BinaryOpKind};
use super::span::Span;

pub struct SymbolId(usize);

pub struct HirId(usize);

pub struct Crate{

}

pub struct Stmt{

}

pub enum StmtKind{

}

pub struct Expr<'hir>{
    span: Span,
    kind: ExprKind<'hir>,
    id: HirId,
}

pub enum ExprKind<'hir>{
    Literal,
    Var(SymbolId),
    FnCall(&'hir Expr<'hir>, &'hir [Expr<'hir>]),
    Unary(UnaryOpKind, &'hir Expr<'hir>),
    Binary(BinaryOpKind, &'hir Expr<'hir>, &'hir Expr<'hir>),
    While(&'hir Expr<'hir>, &'hir Stmt),
}