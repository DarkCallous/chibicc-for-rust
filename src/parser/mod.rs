pub mod helper;

use crate::ast::*;
use crate::error_handler::*;
use crate::span::*;
use core::panic;

pub struct Parser {
    pub tokens: TokenContainer,
    pub index: usize,
    pub errors: Vec<NextTokenError>,
    pub locals: Vec<Symbol>,
}

impl Parser {
    fn peek(&self) -> &Token {
        &self.tokens[self.index]
    }

    pub fn expect(&self, kind: &TokenKind) -> bool {
        if self.index >= self.tokens.len() {
            return false;
        }
        &self.peek().kind == kind
    }

    pub fn expect_and_eat(&mut self, kind: &TokenKind) -> bool {
        let same = self.expect(kind);
        if same {
            self.bump();
        } else {
            let e = NextTokenError::ExpectedToken {
                expected: kind.clone(),
                found: self.peek().clone(),
            };
            self.errors.push(e);
        }
        same
    }

    pub fn bump(&mut self) {
        self.index += 1
    }

    pub fn eat(&mut self, kind: &TokenKind) -> bool {
        let same = self.expect(kind);
        if same {
            self.bump();
        }
        same
    }
}

impl Parser {
    fn parse_lit_num(&mut self) -> Result<(Span, Lit), NextTokenError> {
        let (span, lit) = match &self.peek().kind {
            TokenKind::Literal(i) if i.kind == LitKind::Integer => (self.peek().span, i.clone()),
            _ => {
                let e: Result<(Span, Lit), NextTokenError> = Err(NextTokenError::WrongType {
                    expected: STR_NUMBER,
                    found: self.peek().clone(),
                });
                self.bump();
                return e;
            }
        };

        self.bump();
        Ok((span, lit))
    }

    fn parse_ident(&mut self) -> Option<(Symbol, Span)> {
        let span = self.peek().span;
        let result = if let TokenKind::Ident(s) = &self.peek().kind {
            Some((s.clone(), span))
        } else {
            None
        };
        if result.is_some() {
            self.bump();
        }
        result
    }

    fn parse_primary(&mut self) -> Expr {
        if self.eat(&TokenKind::LParen) {
            let result = self.parse_expr();
            return if !self.expect_and_eat(&TokenKind::RParen) {
                Expr {
                    kind: ExprKind::Error,
                    span: self.peek().span,
                }
            } else {
                result
            };
        }
        if let Some((sym, span)) = self.parse_ident() {
            return if self.eat(&TokenKind::LParen) {
                if !self.expect_and_eat(&TokenKind::RParen) {
                    Expr {
                        kind: ExprKind::Error,
                        span,
                    }
                } else {
                    Expr {
                        kind: ExprKind::FnCall(sym),
                        span,
                    }
                }
            } else {
                self.locals.push(sym.clone());
                Expr {
                    kind: ExprKind::Var(sym),
                    span,
                }
            };
        }
        let ex = self.parse_lit_num();
        match ex {
            Ok((span, kind)) => {
                let kind = kind.clone();
                Expr {
                    kind: ExprKind::Literal(kind),
                    span,
                }
            }
            Err(e) => {
                let error_node = e.gen_error_expr();
                self.errors.push(e);
                error_node
            }
        }
    }

    fn parse_unary(&mut self) -> Expr {
        let result;
        let span = self.peek().span;
        if self.eat(&TokenKind::Add) {
            result = Expr {
                kind: ExprKind::Unary(UnaryOpKind::Pos, Box::new(self.parse_primary())),
                span,
            }
        } else if self.eat(&TokenKind::Sub) {
            result = Expr {
                kind: ExprKind::Unary(UnaryOpKind::Neg, Box::new(self.parse_primary())),
                span,
            }
        } else {
            result = self.parse_primary();
        }
        result
    }

    fn parse_mul(&mut self) -> Expr {
        let mut node = self.parse_unary();
        while self.index < self.tokens.len() {
            let span = self.peek().span;
            let op = match true {
                _ if self.eat(&TokenKind::Mul) => BinaryOpKind::Mul,
                _ if self.eat(&TokenKind::Div) => BinaryOpKind::Div,
                _ => break,
            };
            node = Expr {
                span,
                kind: ExprKind::Binary(op, Box::new(node), Box::new(self.parse_unary())),
            }
        }
        node
    }

    pub fn parse_add(&mut self) -> Expr {
        let mut node = self.parse_mul();
        while self.index < self.tokens.len() {
            let span = self.peek().span;
            let op = match true {
                _ if self.eat(&TokenKind::Add) => BinaryOpKind::Add,
                _ if self.eat(&TokenKind::Sub) => BinaryOpKind::Sub,
                _ => break,
            };
            node = Expr {
                span,
                kind: ExprKind::Binary(op, Box::new(node), Box::new(self.parse_mul())),
            }
        }
        node
    }

    pub fn parse_rational(&mut self) -> Expr {
        let mut node = self.parse_add();
        while self.index < self.tokens.len() {
            let span = self.peek().span;
            let op = match true {
                _ if self.eat(&TokenKind::Ge) => BinaryOpKind::Ge,
                _ if self.eat(&TokenKind::Gt) => BinaryOpKind::Gt,
                _ if self.eat(&TokenKind::Le) => BinaryOpKind::Le,
                _ if self.eat(&TokenKind::Lt) => BinaryOpKind::Lt,
                _ => break,
            };
            node = Expr {
                span,
                kind: ExprKind::Binary(op, Box::new(node), Box::new(self.parse_add())),
            }
        }
        node
    }

    pub fn parse_equality(&mut self) -> Expr {
        let mut node = self.parse_rational();
        while self.index < self.tokens.len() {
            let span = self.peek().span;
            let op = match true {
                _ if self.eat(&TokenKind::EqEq) => BinaryOpKind::EqEq,
                _ if self.eat(&TokenKind::Ne) => BinaryOpKind::Ne,
                _ => break,
            };
            node = Expr {
                span,
                kind: ExprKind::Binary(op, Box::new(node), Box::new(self.parse_rational())),
            }
        }
        node
    }

    pub fn parse_assign(&mut self) -> Expr {
        let mut node = self.parse_equality();

        if self.eat(&TokenKind::Eq) {
            let span = self.peek().span;
            node = Expr {
                span,
                kind: ExprKind::Assign(Box::new(node), Box::new(self.parse_assign())),
            };
        }
        node
    }

    pub fn parse_expr(&mut self) -> Expr {
        self.parse_assign()
    }

    pub fn parse_exprstmt(&mut self) -> Option<Expr> {
        if self.eat(&TokenKind::Semi) {
            return None;
        }
        let result = self.parse_expr();
        Some(if !self.expect_and_eat(&TokenKind::Semi) {
            Expr {
                kind: ExprKind::Error,
                span: self.peek().span,
            }
        } else {
            result
        })
    }

    pub fn parse_compoundstmt(&mut self) -> Stmt {
        let mut stmts = Vec::new();
        while !self.eat(&TokenKind::RBrace) {
            stmts.push(self.parse_stmt());
        }
        Stmt::Block(stmts)
    }

    pub fn parse_stmt(&mut self) -> Stmt {
        if self.eat(&TokenKind::LBrace) {
            self.parse_compoundstmt()
        } else if self.eat(&TokenKind::Keyword(KeywordKind::Return)) {
            let result = Stmt::Return(Box::new(self.parse_expr()));
            if !self.eat(&TokenKind::Semi) {
                panic!("missing ;");
            }
            result
        } else if self.eat(&TokenKind::Keyword(KeywordKind::If)) {
            if !self.eat(&TokenKind::LParen) {
                panic!("missing (")
            }
            let condition = self.parse_expr();
            if !self.eat(&TokenKind::RParen) {
                panic!("missing )")
            }
            let ops = self.parse_stmt();
            let else_ops = if self.eat(&TokenKind::Keyword(KeywordKind::Else)) {
                Some(self.parse_stmt())
            } else {
                None
            };
            Stmt::If(Box::new(condition), Box::new(ops), Box::new(else_ops))
        } else if self.eat(&TokenKind::Keyword(KeywordKind::While)) {
            if !self.eat(&TokenKind::LParen) {
                panic!("missing (")
            }
            let condition = self.parse_expr();
            if !self.eat(&TokenKind::RParen) {
                panic!("missing )")
            }
            let ops = self.parse_stmt();
            Stmt::While(Box::new(condition), Box::new(ops))
        } else if self.eat(&TokenKind::Keyword(KeywordKind::For)) {
            if !self.eat(&TokenKind::LParen) {
                panic!("missing (")
            }
            let ini = self.parse_exprstmt();
            let cond = self.parse_exprstmt();
            let incr = if !self.eat(&TokenKind::RParen) {
                let incr = self.parse_expr();
                if !self.eat(&TokenKind::RParen) {
                    panic!("missing )")
                }
                Some(incr)
            } else {
                None
            };
            let ops = self.parse_stmt();
            Stmt::For(Box::new(ini), Box::new(cond), Box::new(incr), Box::new(ops))
        } else {
            self.parse_exprstmt()
                .map(|expr| Stmt::ExprStmt(Box::new(expr)))
                .unwrap_or(Stmt::Null)
        }
    }

    pub fn parse_crate(&mut self) -> Crate {
        let mut stmts = Vec::new();
        while self.index < self.tokens.len() {
            stmts.push(self.parse_stmt());
        }
        Crate { stmts }
    }
}
