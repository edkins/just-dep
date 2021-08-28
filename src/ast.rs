use num_bigint::BigInt;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Script {
    pub declaration_order: Vec<String>,
    pub funcs: HashMap<String, Func>,
}

#[derive(Clone, Debug)]
pub struct Func {
    pub args: Vec<(String, Expr)>,
    pub ret: Expr,
    pub body: Expr,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Expr {
    Int(BigInt),
    Var(String),
    Call(String, Vec<Expr>),
    Array(Vec<Expr>),
}
