use num_bigint::BigInt;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Script {
    pub funcs: HashMap<String, Func>,
}

#[derive(Clone, Debug)]
pub struct Func {
    pub args: Vec<(String, Expr)>,
    pub ret: Expr,
    pub body: Expr,
}

#[derive(Clone, Debug)]
pub enum Expr {
    Int(BigInt),
    Var(String),
    Call(String, Vec<Expr>),
}
