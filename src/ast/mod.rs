use crate::tokenizer::*;
use crate::span::*;

enum Stmt{
    ExprStmt(),
    Return(),
    Block(Vec<Stmt>),
}

pub enum BinaryOpKind{
    Add,
    Sub,
    Mul,
    Div,
}

pub enum ExprKind{
    Literal(Lit), 
    Binary(BinaryOpKind, Box<Expr>, Box<Expr>),
    Error,
}

pub struct Expr{
    pub kind: ExprKind,
    pub span: Span,
}


