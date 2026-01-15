pub mod helper;

use crate::tokenizer::*;
use crate::ast::*;
use crate::error_handler::*;
use crate::span::*;

pub struct Parser{
    pub tokens: TokenContainer,
    pub index: usize,
    pub errors: Vec<NextTokenError>,
}

impl Parser{
    pub fn expect(&self, kind: &TokenKind) -> bool {
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

    fn parse_lit_num<'a>(&'a mut self) -> Result<(Span, Lit), NextTokenError> {
        let (span, lit) = match &self.peek().kind {
            TokenKind::Literal(i) if i.kind == LitKind::Integer => (self.peek().span, i.clone()),
            _ => {
                let e: Result<(Span, Lit), NextTokenError> = 
                    Err(NextTokenError::WrongType{expected: STR_NUMBER, found: (self.peek()).clone()});
                self.bump();
                return e;
            }
        };
        
        self.bump();
        Ok((span, lit))
    }

    fn parse_primary(&mut self) -> Expr{
        if self.eat(&TokenKind::LParen){
            let result = self.parse_expr();
            self.eat(&TokenKind::RParen);
            return result;
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
        let mut node = self.parse_primary();
        while self.index < self.tokens.len(){
            let span = self.peek().span;
            if self.eat(&TokenKind::Mul){
                node = Expr{span, kind: ExprKind::Binary(BinaryOpKind::Mul, Box::new(node), Box::new(self.parse_primary()))}
            }   
            else if self.eat(&TokenKind::Div){
                node = Expr{span, kind: ExprKind::Binary(BinaryOpKind::Div, Box::new(node), Box::new(self.parse_primary()))}
            }   
            else{
                break;
            }
        } 
        node
    }

    pub fn parse_expr(&mut self) -> Expr{
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
}