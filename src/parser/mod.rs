pub mod helper;

use crate::ast::*;
use crate::error_handler::*;
use crate::span::*;
use core::panic;

pub struct Parser {
    pub tokens: TokenContainer,
    pub index: usize,
    pub errors: Vec<NextTokenError>,
    pub node_cnt: usize,
}

fn is_typename(tk: &TokenKind)->bool{
    match tk{
        TokenKind::Keyword(kwk) if kwk == &KeywordKind::Int => true,
        _ => false,
    }
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
    fn next_expr(&mut self, kind: ExprKind, span: Span) -> Expr {
        let exp = Expr {
            id: self.node_cnt,
            kind,
            span,
        };
        self.node_cnt += 1;
        exp
    }

    fn next_stmt(&mut self, kind: StmtKind) -> Stmt{
        let stmt = Stmt {
            id: self.node_cnt,
            kind,
        };
        self.node_cnt += 1;
        stmt
    }

    fn next_declarator(&mut self, ptr: Option<Box<PointerDecl>>, direct: DirectDeclatator) -> Declarator{
        let decl = Declarator {
            id: self.node_cnt,
            ptr,
            direct,
        };
        self.node_cnt += 1;
        decl
    }

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

    fn parse_params(&mut self) -> Option<Vec<Expr>> {
        let mut result = vec![];
        if self.eat(&TokenKind::RParen) {
            return Some(result);
        }
        loop {
            let expr = self.parse_expr();
            result.push(expr);
            if !self.eat(&TokenKind::Comma) {
                break;
            }
        }
        if self.eat(&TokenKind::RParen) {
            Some(result)
        } else {
            None
        }
    }

    fn parse_primary(&mut self) -> Expr {
        if self.eat(&TokenKind::LParen) {
            let mut result = self.parse_expr();
            if !self.expect_and_eat(&TokenKind::RParen) {
                result.kind = ExprKind::Error;
            }
            return result;
        }
        if let Some((sym, span)) = self.parse_ident() {
            return if self.eat(&TokenKind::LParen) {
                let params = self.parse_params();
                let kind = if let Some(params) = params {
                    ExprKind::FnCall(sym, params)
                } else {
                    ExprKind::Error
                };
                self.next_expr(kind, span)
            } else {
                self.next_expr(ExprKind::Var(sym), span)
            };
        }
        let ex = self.parse_lit_num();
        match ex {
            Ok((span, kind)) => {
                let kind = kind.clone();
                self.next_expr(ExprKind::Literal(kind), span)
            }
            Err(e) => {
                let error_node = e.gen_error_expr(&mut self.node_cnt);
                self.errors.push(e);
                error_node
            }
        }
    }

    fn parse_unary(&mut self) -> Expr {
        let span = self.peek().span;
        let op = match true {
            _ if self.eat(&TokenKind::Add) => Some(UnaryOpKind::Pos),
            _ if self.eat(&TokenKind::Sub) => Some(UnaryOpKind::Neg),
            _ if self.eat(&TokenKind::And) => Some(UnaryOpKind::AddrOf),
            _ if self.eat(&TokenKind::Mul) => Some(UnaryOpKind::Deref),
            _ => None,
        };
        if let Some(op) = op {
            let inner = self.parse_unary();
            self.next_expr(ExprKind::Unary(op, Box::new(inner)), span)
        } else {
            self.parse_primary()
        }
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
            let unary_exp = self.parse_unary();
            node = self.next_expr(
                ExprKind::Binary(op, Box::new(node), Box::new(unary_exp)),
                span,
            );
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
            let mul_expr = self.parse_mul();
            node = self.next_expr(
                ExprKind::Binary(op, Box::new(node), Box::new(mul_expr)),
                span,
            );
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
            let add_expr = self.parse_add();
            node = self.next_expr(
                ExprKind::Binary(op, Box::new(node), Box::new(add_expr)),
                span,
            );
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
            let rational_expr = self.parse_rational();
            node = self.next_expr(
                ExprKind::Binary(op, Box::new(node), Box::new(rational_expr)),
                span,
            );
        }
        node
    }

    pub fn parse_assign(&mut self) -> Expr {
        let mut node = self.parse_equality();

        if self.eat(&TokenKind::Eq) {
            let span = self.peek().span;
            let assign_expr = self.parse_assign();
            node = self.next_expr(
                ExprKind::Assign(Box::new(node), Box::new(assign_expr)),
                span,
            );
        }
        node
    }

    pub fn parse_expr(&mut self) -> Expr {
        self.parse_assign()
    }

    pub fn parse_exprstmt(&mut self) -> Option<Box<Expr>> {
        if self.eat(&TokenKind::Semi) {
            return None;
        }
        let result = self.parse_expr();
        let result = if !self.expect_and_eat(&TokenKind::Semi) {
            self.next_expr(ExprKind::Error, self.peek().span)
        } else {
            result
        };
        Some(Box::new(result))
    }

    pub fn parse_declaration(&mut self) -> Stmt{
        let spec = self.parse_decl_spec();
        let mut decls = vec![];
        if !self.expect(&TokenKind::Semi){
            decls.push(self.parse_var_decl());
            while self.eat(&TokenKind::Comma){
                decls.push(self.parse_var_decl());
            }
        }
        self.expect_and_eat(&TokenKind::Semi);
        self.next_stmt(StmtKind::Decl(spec, decls))
    }

    pub fn parse_decl_spec(&mut self) -> DeclSpec{
        self.eat(&TokenKind::Keyword(KeywordKind::Int));
        DeclSpec {  }
    }

    pub fn parse_var_decl(&mut self) -> VarDecl{
        let declarator  = self.parse_declarator();
        let init = if self.eat(&TokenKind::Eq){
            Some(Box::new(self.parse_expr()))
        } else {
            None
        };
        VarDecl { declarator, init}
    }

    pub fn parse_declarator(&mut self) -> Declarator{
        let direct = self.parse_direct_decl();
        self.next_declarator(None, direct)
    }

    pub fn parse_pointer_decl(&mut self)-> PointerDecl{
        PointerDecl { inner: None }
    }

    pub fn parse_direct_decl(&mut self) -> DirectDeclatator{
        if let Some((sym, _)) = self.eat_ident(){
            DirectDeclatator::Ident(sym)
        }
        else{
            panic!("no direct decl found")
        }
    }

    pub fn eat_ident(&mut self) -> Option<(Symbol, Span)>{
        let result = if let TokenKind::Ident(sym) = &self.peek().kind{
            Some((sym.clone(), self.peek().span))
        } else{
            None
        };
        if result.is_some(){
            self.bump();
        }
        result
    }

    pub fn parse_compoundstmt(&mut self) -> Stmt {
        let mut stmts = Vec::new();
        while !self.eat(&TokenKind::RBrace) {
            stmts.push( 
                if is_typename(&self.peek().kind){
                    self.parse_declaration()
                } else {
                    self.parse_stmt()
                }
            );
        }
        self.next_stmt(StmtKind::Block(stmts))
    }

    pub fn parse_stmt(&mut self) -> Stmt {
        if self.eat(&TokenKind::LBrace) {
            self.parse_compoundstmt()
        } else if self.eat(&TokenKind::Keyword(KeywordKind::Return)) {
            let result = StmtKind::Return(Box::new(self.parse_expr()));
            if !self.eat(&TokenKind::Semi) {
                panic!("missing ;");
            }
            self.next_stmt(result)
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
                Some(Box::new(self.parse_stmt()))
            } else {
                None
            };
            self.next_stmt(StmtKind::If(Box::new(condition), Box::new(ops), else_ops))
        } else if self.eat(&TokenKind::Keyword(KeywordKind::While)) {
            if !self.eat(&TokenKind::LParen) {
                panic!("missing (")
            }
            let condition = self.parse_expr();
            if !self.eat(&TokenKind::RParen) {
                panic!("missing )")
            }
            let ops = self.parse_stmt();
            self.next_stmt(StmtKind::While(Box::new(condition), Box::new(ops)))
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
                Some(Box::new(incr))
            } else {
                None
            };
            let ops = self.parse_stmt();
            self.next_stmt(StmtKind::For(ini, cond, incr, Box::new(ops)))
        } else {
            let kind = self.parse_exprstmt()
                .map(StmtKind::ExprStmt)
                .unwrap_or(StmtKind::Null);
            self.next_stmt(kind)
        }
    }

    pub fn parse_params_def(&mut self) -> Vec<(Symbol, Ty)> {
        let mut res = vec![];
        if self.eat(&TokenKind::RParen) {
            return res;
        }

        res.push((self.parse_ident().unwrap().0, Ty::Int));
        while !self.eat(&TokenKind::RParen) {
            self.expect_and_eat(&TokenKind::Comma);
            res.push((self.parse_ident().unwrap().0, Ty::Int));
        }
        res
    }

    pub fn parse_fn(&mut self) -> Fn {
        let (name, _) = self.parse_ident().unwrap();
        self.expect_and_eat(&TokenKind::LParen);
        let params = self.parse_params_def();
        self.expect_and_eat(&TokenKind::LBrace);
        let body = self.parse_compoundstmt();
        Fn {
            name: name.clone(),
            body,
            params,
        }
    }

    pub fn parse_crate(&mut self) -> Crate {
        let mut fns = vec![];
        while self.index < self.tokens.len() {
            fns.push(self.parse_fn());
        }
        Crate { fns }
    }
}
