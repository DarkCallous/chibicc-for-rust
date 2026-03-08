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
    pub spec: DeclSpec,
    pub declarator: Declarator,
    pub body: Stmt,
}

pub struct DeclSpec{

}

pub struct PointerDecl{
    pub inner: Option<Box<PointerDecl>>,
}

pub struct ParamDecl {
    pub spec: DeclSpec,
    pub declarator: Option<Declarator>,
}

pub enum DirectDeclarator{
    Ident(Symbol),
    Paren(Box<Declarator>),

    Func {
        inner: Box<DirectDeclarator>,
        params: Vec<ParamDecl>,   
    },

    Array {
        inner: Box<DirectDeclarator>,
        size: Option<Box<Expr>>,
    },
}

pub struct Declarator{
    pub ptr: Option<Box<PointerDecl>>,
    pub direct: DirectDeclarator,
    pub id: NodeId,
}

pub struct VarDecl{
    pub declarator: Declarator,
    pub init: Option<Box<Expr>>,
}

pub enum StmtKind {
    Block(Vec<Stmt>),
    ExprStmt(Box<Expr>),
    Return(Box<Expr>),
    If(Box<Expr>, Box<Stmt>, Option<Box<Stmt>>),
    While(Box<Expr>, Box<Stmt>),
    For(
        Option<Box<Expr>>,
        Option<Box<Expr>>,
        Option<Box<Expr>>,
        Box<Stmt>,
    ),
    Decl(DeclSpec, Vec<VarDecl>),
    Null,
}

pub struct Stmt{
    pub kind: StmtKind,
    pub id: NodeId,
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

#[derive(PartialEq)]
pub enum UnaryOpKind {
    Pos,
    Neg,
    AddrOf,
    Deref,
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

impl Declarator {
    pub fn function_name(&self) -> Option<&Symbol> {
        fn peel(d: &DirectDeclarator) -> Option<&Symbol> {
            match d {
                DirectDeclarator::Ident(sym) => Some(sym),

                DirectDeclarator::Paren(inner_decl) => {
                    peel(&inner_decl.direct)
                }

                DirectDeclarator::Func { inner, .. } => {
                    peel(inner)
                }

                DirectDeclarator::Array { inner, .. } => {
                    peel(inner)
                }
            }
        }

        peel(&self.direct)
    }
}
