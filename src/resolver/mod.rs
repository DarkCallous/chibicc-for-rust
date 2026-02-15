use crate::ast::*;
use std::collections::HashMap;

use super::tokenizer::*;

pub type ObjId = usize;

#[derive(Default)]
pub struct ResolvedCrate {
    pub expr_resolutions: HashMap<NodeId, ObjId>,
    pub objs: Vec<Obj>,
    pub fn_info: HashMap<Symbol, FnInfo>,
}

#[derive(Default)]
pub struct ScopeFrame {
    pub ord_map: HashMap<Symbol, ObjId>,
    pub tagged_map: HashMap<Symbol, ObjId>,
}

pub enum ObjKind {
    Local,
    Param,
    Global,
    Func,
    EnumConst,
}

pub struct FnInfo {
    pub fn_id: ObjId,
    pub params: Vec<ObjId>,
    pub locals: Vec<ObjId>,
}

impl FnInfo {
    pub fn new(fn_id: ObjId) -> FnInfo {
        FnInfo {
            fn_id,
            params: vec![],
            locals: vec![],
        }
    }
}

pub struct Obj {
    pub id: ObjId,
    pub name: Symbol,
    pub kind: ObjKind,
}

pub struct Resolver {
    scopes: Vec<ScopeFrame>,
    pub resolved: ResolvedCrate,
    obj_cnt: usize,
    operating_fn: Option<FnInfo>,
}

impl Default for Resolver {
    fn default() -> Self {
        Resolver::new()
    }
}

impl Resolver {
    pub fn new() -> Resolver {
        Resolver {
            scopes: vec![ScopeFrame::default()],
            resolved: ResolvedCrate::default(),
            obj_cnt: 0,
            operating_fn: None,
        }
    }

    pub fn resolve(&mut self, source: &Crate) {
        self.resolved.fn_info = source
            .fns
            .iter()
            .map(|func| (func.name.clone(), self.resolve_fn(func)))
            .collect::<HashMap<_, _>>();
    }

    pub fn resolve_fn(&mut self, func: &Fn) -> FnInfo {
        let id = self.declare_fn(func.name.clone());
        self.operating_fn = Some(FnInfo::new(id));
        let fn_frame = ScopeFrame::default();
        self.scopes.push(fn_frame);
        for (name, ty) in &func.params {
            self.declare_param(name.clone());
        }

        for stmt in &func.stmts {
            self.resolve_stmt(stmt);
        }
        self.scopes.pop();
        self.operating_fn.take().unwrap()
    }

    pub fn resolve_stmt(&mut self, stmt: &Stmt) {
        match &stmt {
            Stmt::Block(stmts) => {
                self.scopes.push(ScopeFrame::default());
                for internal_stmt in stmts {
                    self.resolve_stmt(internal_stmt);
                }
                self.scopes.pop();
            }
            Stmt::ExprStmt(expr) => {
                self.resolve_expr(expr.as_ref());
            }
            Stmt::Return(expr) => {
                self.resolve_expr(expr.as_ref());
            }
            Stmt::For(init, cond, incr, stmt) => {
                init.as_ref()
                    .as_ref()
                    .inspect(|expr| self.resolve_expr(expr));
                cond.as_ref()
                    .as_ref()
                    .inspect(|expr| self.resolve_expr(expr));
                incr.as_ref()
                    .as_ref()
                    .inspect(|expr| self.resolve_expr(expr));
                self.resolve_stmt(stmt);
            }
            Stmt::While(cond, stmt) => {
                self.resolve_expr(cond.as_ref());
                self.resolve_stmt(stmt);
            }
            Stmt::If(cond, if_stmt, else_stmt) => {
                self.resolve_expr(cond.as_ref());
                self.resolve_stmt(if_stmt);
                else_stmt
                    .as_ref()
                    .as_ref()
                    .inspect(|stmt| self.resolve_stmt(stmt));
            }
            Stmt::Null => (),
        }
    }

    pub fn resolve_expr(&mut self, expr: &Expr) {
        match &expr.kind {
            ExprKind::Assign(a, b) => {
                self.resolve_expr(a.as_ref());
                self.resolve_expr(b.as_ref());
            }
            ExprKind::Binary(_, a, b) => {
                self.resolve_expr(a.as_ref());
                self.resolve_expr(b.as_ref());
            }
            ExprKind::FnCall(sym, params) => {
                let id = expr.id;
                let obj = self
                    .lookup(sym)
                    .expect("identfier must be declared before use");
                self.resolved.expr_resolutions.insert(id, obj);
                for expr in params {
                    self.resolve_expr(expr);
                }
            }
            ExprKind::Unary(_, expr) => {
                self.resolve_expr(expr.as_ref());
            }
            ExprKind::Var(sym) => {
                let id = expr.id;
                let obj = self
                    .lookup(sym)
                    .expect("identfier must be declared before use");
                self.resolved.expr_resolutions.insert(id, obj);
            }
            ExprKind::Literal(_) | ExprKind::Error => (),
        }
    }

    pub fn declare_local(&mut self, name: Symbol) -> ObjId {
        let obj = Obj {
            id: self.obj_cnt,
            name,
            kind: ObjKind::Local,
        };
        let scope = self
            .scopes
            .last_mut()
            .expect("internal declare must in scope");
        scope.ord_map.insert(obj.name.clone(), obj.id);
        let id = obj.id;
        self.resolved.objs.push(obj);
        self.obj_cnt += 1;
        self.operating_fn.as_mut().unwrap().locals.push(id);
        id
    }

    pub fn declare_param(&mut self, name: Symbol) -> ObjId {
        let obj = Obj {
            id: self.obj_cnt,
            name,
            kind: ObjKind::Param,
        };
        let scope = self
            .scopes
            .last_mut()
            .expect("internal declare must in scope");
        scope.ord_map.insert(obj.name.clone(), obj.id);
        let id = obj.id;
        self.resolved.objs.push(obj);
        self.obj_cnt += 1;
        self.operating_fn.as_mut().unwrap().params.push(id);
        id
    }

    pub fn declare_fn(&mut self, name: Symbol) -> ObjId {
        let obj = Obj {
            id: self.obj_cnt,
            name,
            kind: ObjKind::Func,
        };
        self.scopes[0].ord_map.insert(obj.name.clone(), obj.id);
        let id = obj.id;
        self.resolved.objs.push(obj);
        self.obj_cnt += 1;
        id
    }

    pub fn lookup(&self, id: &Symbol) -> Option<ObjId> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.ord_map.get(id).copied())
    }
}
