use num_bigint::BigInt;

#[derive(Clone, Debug)]
pub struct Script {
    pub decls: Vec<(String, Decl)>,
}

#[derive(Clone, Debug)]
pub struct Decl {
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
