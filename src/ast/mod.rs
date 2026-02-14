use crate::span::*;
use crate::tokenizer::*;

pub type NodeId = usize;

pub struct Crate {
    pub fns: Vec<Fn>,
}

pub enum Ty {
    Int,
}

pub struct Fn {
    pub name: Symbol,
    pub params: Vec<(Symbol, Ty)>,
    pub stmts: Vec<Stmt>,
}

pub enum Stmt {
    Block(Vec<Stmt>),
    ExprStmt(Box<Expr>),
    Return(Box<Expr>),
    If(Box<Expr>, Box<Stmt>, Box<Option<Stmt>>),
    While(Box<Expr>, Box<Stmt>),
    For(
        Box<Option<Expr>>,
        Box<Option<Expr>>,
        Box<Option<Expr>>,
        Box<Stmt>,
    ),
    Null,
}

pub enum BinaryOpKind {
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

pub enum UnaryOpKind {
    Pos,
    Neg,
}

pub enum ExprKind {
    Literal(Lit),
    Binary(BinaryOpKind, Box<Expr>, Box<Expr>),
    Unary(UnaryOpKind, Box<Expr>),
    Assign(Box<Expr>, Box<Expr>),
    Var(Symbol),
    FnCall(Symbol, Vec<Expr>),
    Error,
}

pub struct Expr {
    pub id: NodeId,
    pub kind: ExprKind,
    pub span: Span,
}

impl BinaryOpKind {
    pub fn is_compartor(&self) -> bool {
        use BinaryOpKind::*;
        matches!(&self, Ne | EqEq | Ge | Gt | Le | Lt)
    }
}
