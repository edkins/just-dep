use std::collections::HashMap;
use num_bigint::{BigInt,Sign};

use crate::ast::{Expr,Func};

struct CheckedFunc {
    args: Vec<(String,Expr)>,
    ret: Expr,
}

pub enum TypeError {
    ExpectedArgToBeOfTypeType(String, Expr, Expr),
    DuplicateArgName(String),
    CannotCoerceReturnType(Expr, Expr),
    CannotCoerceArgumentType(String, usize, Expr, Expr, Expr),
    NoSuchFunc(String),
    NoSuchVar(String),
    WrongNumberOfArgs(String, usize, usize),
}

fn check_func(func: &Func, funcs: &HashMap<String, CheckedFunc>) -> Result<CheckedFunc, TypeError> {
    let mut env = HashMap::new();

    for arg in &func.args {
        check_arg_is_of_type_type(&arg.0, &arg.1, funcs, &env)?;
        if env.contains_key(&arg.0) {
            return Err(TypeError::DuplicateArgName(arg.0.clone()));
        }
        env.insert(arg.0.clone(), arg.1.clone());
    }

    let t = check_expr(&func.body, funcs, &env)?;
    if !can_coerce_type(&t, &func.ret, funcs, &env) {
        return Err(TypeError::CannotCoerceReturnType(t, func.ret.clone()));
    }

    Ok(CheckedFunc {
        args: func.args.clone(),
        ret: func.ret.clone(),
    })
}

fn check_arg_is_of_type_type(name: &str, expr: &Expr, funcs: &HashMap<String, CheckedFunc>, env: &HashMap<String, Expr>) -> Result<(), TypeError> {
    let t = check_expr(expr, funcs, env)?;
    let typ = Expr::Var("type".to_owned());
    if can_coerce_type(&t, &typ, funcs, env) {
        Ok(())
    } else {
        Err(TypeError::ExpectedArgToBeOfTypeType(name.to_owned(), expr.clone(), t))
    }
}

fn check_expr(expr: &Expr, funcs: &HashMap<String, CheckedFunc>, env: &HashMap<String, Expr>) -> Result<Expr, TypeError> {
    match expr {
        Expr::Int(n) => {
            if n.sign() == Sign::Minus {
                Ok(Expr::Var("int".to_owned()))
            } else {
                Ok(Expr::Var("uint".to_owned()))
            }
        }
        Expr::Var(x) => {
            if let Some(t) = env.get(x) {
                Ok(t.clone())
            } else {
                Err(TypeError::NoSuchVar(x.clone()))
            }
        }
        Expr::Call(f, xs) => {
            if let Some(cf) = funcs.get(f) {
                if cf.args.len() == xs.len() {
                    let ts = xs.iter().map(|x|check_expr(x, funcs, env)).collect::<Result<Vec<_>,_>>()?;
                    let mut var_mapping = HashMap::new();
                    for i in 0..ts.len() {
                        let t1 = cf.args[i].1.map_vars(&var_mapping)?;
                        if !can_coerce_type(&ts[i], &t1, funcs, env) {
                            return Err(TypeError::CannotCoerceArgumentType(f.clone(), i, xs[i].clone(), ts[i].clone(), t1));
                        }
                        if var_mapping.contains_key(&cf.args[i].0) {
                            return Err(TypeError::DuplicateArgName(cf.args[i].0.clone()));
                        }
                        var_mapping.insert(cf.args[i].0.clone(), xs[i].clone());
                    }
                    cf.ret.map_vars(&var_mapping)
                } else {
                    Err(TypeError::WrongNumberOfArgs(f.clone(), cf.args.len(), xs.len()))
                }
            } else {
                Err(TypeError::NoSuchFunc(f.clone()))
            }
        }
        Expr::Array(xs) => {
            let ts = xs.iter().map(|x|check_expr(x, funcs, env)).collect::<Result<Vec<_>,_>>()?;
            Ok(Expr::Call("tuple".to_owned(), vec![Expr::Array(ts)]))
        }
    }
}

impl Expr {
    fn map_vars(&self, var_mapping: &HashMap<String, Expr>) -> Result<Expr,TypeError> {
        match self {
            Expr::Int(_) => Ok(self.clone()),
            Expr::Var(x) => {
                if let Some(y) = var_mapping.get(x) {
                    Ok(y.clone())
                } else {
                    Err(TypeError::NoSuchVar(x.clone()))
                }
            }
            Expr::Call(f, xs) => Ok(Expr::Call(f.clone(), xs.iter().map(|x|x.map_vars(var_mapping)).collect::<Result<_,_>>()?)),
            Expr::Array(xs) => Ok(Expr::Array(xs.iter().map(|x|x.map_vars(var_mapping)).collect::<Result<_,_>>()?)),
        }
    }

    fn is_label(&self, label: &str) -> bool {
        match self {
            Expr::Var(t) => t == label,
            _ => false,
        }
    }

    fn is_list_type(&self) -> Option<&Expr> {
        match self {
            Expr::Call(f, xs) => if f == "list" && xs.len() == 1 {
                Some(&xs[0])
            } else {
                None
            },
            _ => None
        }
    }

    fn is_vector_type(&self) -> Option<(&Expr, &Expr)> {
        match self {
            Expr::Call(f, xs) => if f == "list" && xs.len() == 2 {
                Some((&xs[0], &xs[1]))
            } else {
                None
            },
            _ => None
        }
    }

    fn is_tuple_type(&self) -> Option<&Expr> {
        match self {
            Expr::Call(f, xs) => if f == "tuple" && xs.len() == 1 {
                Some(&xs[0])
            } else {
                None
            },
            _ => None
        }
    }

    fn is_explicit_array(&self) -> Option<&[Expr]> {
        match self {
            Expr::Array(xs) => Some(xs),
            _ => None
        }
    }

    fn is_literal_integer(&self) -> Option<&BigInt> {
        match self {
            Expr::Int(n) => Some(n),
            _ => None
        }
    }
}

/// Returns whether `sub` is known to be coercible to `sup` in the given environment.
///
/// Assumes sub and sup are actually known to be types
///
/// These coercions are possible:
///
/// - t < t
/// - false < t
/// - uint < int
/// - list t0 < list t1           if t0 < t1
/// - vector t0 n < list t1       if t0 < t1
/// - tuple ts < list t1          if each of ts < t1
/// - vector t0 m < vector t1 n   if t0 < t1 and m == n
/// - tuple ts < vector t1 n      if n==length ts and each of ts < t1
/// - vector t0 n < tuple ts      if n==length ts and t0 < each of ts
/// - tuple ts0 < tuple ts1       if length ts0==length ts1 and each of ts0 < corresponding ts1
///
/// Note also that true = vector t 0 = tuple [], but I'm not sure how useful this is in practice
///
fn can_coerce_type(sub: &Expr, sup: &Expr, funcs: &HashMap<String, CheckedFunc>, env: &HashMap<String, Expr>) -> bool {
    if sub == sup || sub.is_label("false") {
        true
    } else if sup.is_label("int") {
        sub.is_label("uint")
    } else if let Some(t1) = sup.is_list_type() {
        if let Some(t0) = sub.is_list_type() {
            can_coerce_type(t0, t1, funcs, env)
        } else if let Some((t0, _n)) = sub.is_vector_type() {
            can_coerce_type(t0, t1, funcs, env)
        } else if let Some(ts_expr) = sub.is_tuple_type() {
            if let Some(ts) = ts_expr.is_explicit_array() {
                ts.iter().all(|t|can_coerce_type(t, t1, funcs, env))
            } else {
                false  // more of a not sure than a false
            }
        } else {
            false
        }
    } else if let Some((t1, n)) = sup.is_vector_type() {
        if let Some((t0, m)) = sub.is_vector_type() {
            can_coerce_type(t0, t1, funcs, env) && can_prove_equal(m, n, funcs, env)
        } else if let Some(ts_expr) = sub.is_tuple_type() {
            if let Some(ts) = ts_expr.is_explicit_array() {
                can_prove_equal_usize(n, ts.len(), funcs, env) && ts.iter().all(|t|can_coerce_type(t, t1, funcs, env))
            } else {
                false
            }
        } else {
            false
        }
    } else if let Some(ts1_expr) = sup.is_tuple_type() {
        if let Some(ts1) = ts1_expr.is_explicit_array() {
            if let Some(ts0_expr) = sub.is_tuple_type() {
                if let Some((t0, n)) = sub.is_vector_type() {
                    can_prove_equal_usize(n, ts1.len(), funcs, env) && ts1.iter().all(|t1|can_coerce_type(t0, t1, funcs, env))
                } else if let Some(ts0) = ts0_expr.is_explicit_array() {
                    ts0.len() == ts1.len() && ts0.iter().zip(ts1).all(|(t0,t1)|can_coerce_type(t0, t1, funcs, env))
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        }
    } else {
        false
    }
}

/// For now, only prove equality if they're written identically
fn can_prove_equal(a: &Expr, b: &Expr, _funcs: &HashMap<String, CheckedFunc>, _env: &HashMap<String, Expr>) -> bool {
    a == b
}

fn can_prove_equal_usize(a: &Expr, b: usize, funcs: &HashMap<String, CheckedFunc>, env: &HashMap<String, Expr>) -> bool {
    can_prove_equal(a, &Expr::Int(b.into()), funcs, env)
}
