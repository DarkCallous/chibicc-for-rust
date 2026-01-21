pub mod helper;

use core::panic;
use crate::ast::*;
use crate::error_handler::*;
use crate::span::*;

pub struct Parser{
    pub tokens: TokenContainer,
    pub index: usize,
    pub errors: Vec<NextTokenError>,
    pub locals: Vec<Symbol>,
}

impl Parser{
    pub fn expect(&self, kind: &TokenKind) -> bool {
        if self.index >= self.tokens.len(){
            return false
        }
        &self.tokens[self.index].kind == kind
    }

    pub fn bump(&mut self){
        self.index += 1
    }

    pub fn eat(&mut self, kind: &TokenKind) -> bool{
        let same = self.expect(&kind);
        if same{
            self.bump();
        }
        same
    }
}

impl Parser{
    fn peek(&self) -> &Token {
        &self.tokens[self.index]
    }

    fn parse_lit_num(&mut self) -> Result<(Span, Lit), NextTokenError> {
        let (span, lit) = match &self.peek().kind {
            TokenKind::Literal(i) if i.kind == LitKind::Integer => (self.peek().span, i.clone()),
            _ => {
                let e: Result<(Span, Lit), NextTokenError> = 
                    Err(NextTokenError::WrongType{expected: STR_NUMBER, found: self.peek().clone()});
                self.bump();
                return e;
            }
        };
        
        self.bump();
        Ok((span, lit))
    }

    fn parse_unary(&mut self) -> Expr{
        let result;
        let span = self.peek().span;
        if self.eat(&TokenKind::Add){
            result = Expr { kind: ExprKind::Unary(UnaryOpKind::Pos, Box::new(self.parse_primary())), span }
        } 
        else if self.eat(&TokenKind::Sub){
            result = Expr { kind: ExprKind::Unary(UnaryOpKind::Neg, Box::new(self.parse_primary())), span }
        } 
        else {
            result = self.parse_primary();    
        }
        result
    }

    fn parse_ident(&mut self) -> Option<Expr>{
        let span = self.peek().span;
        let result = if let TokenKind::Ident(s) = &self.peek().kind{
            Some(Expr { kind: ExprKind::Var(s.clone()), span})
        } else{
            None
        };
        if let Some(Expr{kind: ExprKind::Var(sym), span: _}) = &result{
            if !self.locals.contains(sym){
                self.locals.push(sym.clone());
            }
            self.bump();
        }
        result
    }

    fn parse_primary(&mut self) -> Expr{
        if self.eat(&TokenKind::LParen){
            let result = self.parse_expr();
            self.eat(&TokenKind::RParen);
            return result;
        }
        if let Some(sym) = self.parse_ident(){
            return sym;
        }
        let ex = self.parse_lit_num();
        match ex{
            Ok((span, kind)) =>{
                let kind = kind.clone();
                Expr{kind: ExprKind::Literal(kind), span}
            }
            Err(e) => {
                let span = match &e {
                    NextTokenError::WrongType { found, .. } => found.span,
                };
                self.errors.push(e);
                Expr {
                    kind: ExprKind::Error,
                    span,
                }
            }
        }
    }

    fn parse_mul(&mut self) -> Expr{
        let mut node = self.parse_unary();
        while self.index < self.tokens.len(){
            let span = self.peek().span;
            if self.eat(&TokenKind::Mul){
                node = Expr{span, kind: ExprKind::Binary(BinaryOpKind::Mul, Box::new(node), Box::new(self.parse_unary()))}
            }   
            else if self.eat(&TokenKind::Div){
                node = Expr{span, kind: ExprKind::Binary(BinaryOpKind::Div, Box::new(node), Box::new(self.parse_unary()))}
            }   
            else{
                break;
            }
        } 
        node
    }

    pub fn parse_add(&mut self) -> Expr{
        let mut node = self.parse_mul();
        while self.index < self.tokens.len(){
            let span = self.peek().span;
            if self.eat(&TokenKind::Add){
                node = Expr{span, kind: ExprKind::Binary(BinaryOpKind::Add, Box::new(node), Box::new(self.parse_mul()))}
            }   
            else if self.eat(&TokenKind::Sub){
                node = Expr{span, kind: ExprKind::Binary(BinaryOpKind::Sub, Box::new(node), Box::new(self.parse_mul()))}
            }   
            else{
                break;
            }
        } 
        node
    }

    pub fn parse_rational(&mut self)->Expr{
        let mut node = self.parse_add();
        while self.index < self.tokens.len(){
            let span = self.peek().span;
            if self.eat(&TokenKind::Ge){
                node = Expr{span, kind: ExprKind::Binary(BinaryOpKind::Ge, Box::new(node), Box::new(self.parse_add()))}
            }   
            else if self.eat(&TokenKind::Gt){
                node = Expr{span, kind: ExprKind::Binary(BinaryOpKind::Gt, Box::new(node), Box::new(self.parse_add()))}
            } 
            else if self.eat(&TokenKind::Le){
                node = Expr{span, kind: ExprKind::Binary(BinaryOpKind::Le, Box::new(node), Box::new(self.parse_add()))}
            } 
            else if self.eat(&TokenKind::Lt){
                node = Expr{span, kind: ExprKind::Binary(BinaryOpKind::Lt, Box::new(node), Box::new(self.parse_add()))}
            }   
            else{
                break;
            }
        } 
        node
    }

    pub fn parse_equality(&mut self)->Expr{
        let mut node = self.parse_rational();
        while self.index < self.tokens.len(){
            let span = self.peek().span;
            if self.eat(&TokenKind::EqEq){
                node = Expr{span, kind: ExprKind::Binary(BinaryOpKind::EqEq, Box::new(node), Box::new(self.parse_rational()))}
            }   
            else if self.eat(&TokenKind::Ne){
                node = Expr{span, kind: ExprKind::Binary(BinaryOpKind::Ne, Box::new(node), Box::new(self.parse_rational()))}
            }   
            else{
                break;
            }
        } 
        node
    }

    pub fn parse_assign(&mut self)->Expr{
        let mut node = self.parse_equality();
        
        if self.eat(&TokenKind::Eq){
            let span = self.peek().span;
            node = Expr { span, kind: ExprKind::Assign(Box::new(node), Box::new(self.parse_assign())) };
        }
        node
    }

    pub fn parse_expr(&mut self)->Expr{
        self.parse_assign()
    }

    pub fn parse_stmt(&mut self)->Stmt{
        let stmt =  if self.eat(&TokenKind::Keyword(KeywordKind::Return)){
            Stmt::Return(Box::new(self.parse_expr()))
        }
        else{
            Stmt::ExprStmt(Box::new(self.parse_expr()))
        };
        
        if !self.eat(&TokenKind::Semi){
            panic!("missing ;");
        }
        stmt
    }

    pub fn parse_crate(&mut self)->Crate{
        let mut stmts = Vec::new();
        while self.index < self.tokens.len(){
            stmts.push(self.parse_stmt());
        }
        Crate{
            stmts
        }
    }
}