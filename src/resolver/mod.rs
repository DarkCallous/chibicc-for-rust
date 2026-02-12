use std::collections::HashMap;
use crate::ast::Crate;

use super::tokenizer::*;

type ObjId = usize;

pub struct ScopeFrame{
    pub ord_map: HashMap<Symbol, ObjId>,
    pub tagged_map: HashMap<Symbol, ObjId>,
}

pub struct Resolver{
    scopes: Vec<ScopeFrame>,
}

impl Resolver{
    pub fn new()->Resolver{
        Resolver { scopes: vec![] }
    }

    pub fn resolve(source: Crate){
        
    }

}