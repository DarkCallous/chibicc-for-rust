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
    EqEq,
    Ne,
    Le,
    Lt,
    Ge,
    Gt,
}

pub enum UnaryOpKind{
    Pos,
    Neg,
}

pub enum ExprKind{
    Literal(Lit), 
    Binary(BinaryOpKind, Box<Expr>, Box<Expr>),
    Unary(UnaryOpKind, Box<Expr>),
    Error,
}

pub struct Expr{
    pub kind: ExprKind,
    pub span: Span,
}

impl BinaryOpKind{
    pub fn is_compartor(&self)->bool{
        use BinaryOpKind::*;
        match &self{
            Ne | EqEq | Ge | Gt | Le | Lt => true,
            _ => false
        }
    }
}


