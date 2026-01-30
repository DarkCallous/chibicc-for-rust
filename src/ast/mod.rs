use crate::tokenizer::*;
use crate::span::*;

pub struct Crate{
    pub stmts: Vec<Stmt>,
}

pub enum Stmt{
    Block(Vec<Stmt>),
    ExprStmt(Box<Expr>),
    Return(Box<Expr>),
    If(Box<Expr>, Box<Stmt>, Box<Option<Stmt>>),
    While(Box<Expr>, Box<Stmt>),
    For(Box<Option<Expr>>, Box<Option<Expr>>, Box<Option<Expr>>, Box<Stmt>),
    Null,
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
    Assign(Box<Expr>, Box<Expr>),
    Var(Symbol),
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


