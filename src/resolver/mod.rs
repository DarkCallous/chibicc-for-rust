use crate::ast::Crate;
use std::collections::HashMap;

use super::tokenizer::*;

type ObjId = usize;

pub struct ScopeFrame {
    pub ord_map: HashMap<Symbol, ObjId>,
    pub tagged_map: HashMap<Symbol, ObjId>,
}

pub struct Resolver {
    scopes: Vec<ScopeFrame>,
}

impl Default for Resolver {
    fn default() -> Self {
        Resolver::new()
    }
}

impl Resolver {
    pub fn new() -> Resolver {
        Resolver { scopes: vec![] }
    }

    pub fn resolve(source: Crate) {
        for stmt in source.stmts {}
    }
}
