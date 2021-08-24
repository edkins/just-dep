use std::collections::HashMap;
use std::fmt;
use num_bigint::{BigInt, Sign};
use num_traits::cast::ToPrimitive;

use crate::ast::{Func, Script, Expr};

#[derive(Clone, Debug)]
pub enum Type {
    Bool,
    Int,
    Uint,
    String,
    List(Box<Type>),
    Vector(Box<Type>, usize),
    Tuple(Vec<Type>),
    Type,
}

#[derive(Clone, Debug)]
pub enum Val {
    Bool(bool),
    Int(BigInt),
    String(String),
    Array(Vec<Val>),
    Type(Type),
}

#[derive(Debug)]
pub enum EvalError {
    WrongNumberOfArgs(String, usize, usize),
    NoSuchVar(String),
    NoSuchFunc(String),
    ArgTypeEvalError(String, Box<EvalError>),
    ArgTypeNotType(String),
    ArgIsWrongType(String, Type, Val),
    RetTypeEvalError(Box<EvalError>),
    RetTypeNotType,
    ResultIsWrongType,
    Overflow,
    UnexpectedError,
}

impl Val {
    fn unwrap_usize(&self) -> Result<usize, EvalError> {
        match self {
            Val::Int(i) => match i.to_usize() {
                None => Err(EvalError::Overflow),
                Some(n) => Ok(n),
            },
            _ => Err(EvalError::UnexpectedError),
        }
    }

    fn unwrap_type(&self) -> Result<Type, EvalError> {
        match self {
            Val::Type(t) => Ok(t.clone()),
            _ => Err(EvalError::UnexpectedError),
        }
    }

    fn unwrap_array(&self) -> Result<Vec<Val>, EvalError> {
        match self {
            Val::Array(xs) => Ok(xs.clone()),
            _ => Err(EvalError::UnexpectedError),
        }
    }

    fn unwrap_array_of_types(&self) -> Result<Vec<Type>, EvalError> {
        self.unwrap_array()?.iter().map(Val::unwrap_type).collect()
    }
}

impl Script {
    fn lookup_value(&self, name: &str, env: &HashMap<String, Val>) -> Result<Val, EvalError> {
        match name {
            "true" => Ok(Val::Bool(true)),
            "false" => Ok(Val::Bool(false)),
            "bool" => Ok(Val::Type(Type::Bool)),
            "int" => Ok(Val::Type(Type::Int)),
            "uint" => Ok(Val::Type(Type::Uint)),
            "string" => Ok(Val::Type(Type::String)),
            "type" => Ok(Val::Type(Type::Type)),
            _ => {
                match env.get(name) {
                    None => Err(EvalError::NoSuchVar(name.to_owned())),
                    Some(x) => Ok(x.clone()),
                }
            }
        }
    }

    fn lookup_fn(&self, name: &str) -> Result<Func, EvalError> {
        match name {
            "list" => Ok(Func {
                args: vec![("t".to_owned(),Expr::Var("type".to_owned()))],
                ret: Expr::Var("type".to_owned()),
                body: Expr::Var("".to_owned()), // dummy nonsense value for body
            }),
            "vector" => Ok(Func {
                args: vec![("t".to_owned(),Expr::Var("type".to_owned())), ("n".to_owned(), Expr::Var("uint".to_owned()))],
                ret: Expr::Var("type".to_owned()),
                body: Expr::Var("".to_owned()),
            }),
            "tuple" => Ok(Func {
                args: vec![("ts".to_owned(),Expr::Call("list".to_owned(), vec![Expr::Var("type".to_owned())]))],
                ret: Expr::Var("type".to_owned()),
                body: Expr::Var("".to_owned()),
            }),
            _ => {
                match self.funcs.get(name) {
                    None => Err(EvalError::NoSuchFunc(name.to_owned())),
                    Some(f) => Ok(f.clone()),   // TODO: avoid clone
                }
            }
        }
    }

    fn has_type(&self, value: &Val, typ: &Type) -> bool {
        match (value, typ) {
            (Val::Bool(_), Type::Bool) | (Val::Int(_), Type::Int) | (Val::String(_), Type::String) | (Val::Type(_), Type::Type) => true,
            (Val::Int(n), Type::Uint) => n.sign() != Sign::Minus,
            (Val::Array(xs), Type::List(t)) => xs.iter().all(|x| self.has_type(x, t)),
            (Val::Array(xs), Type::Vector(t,n)) => xs.len() == *n && xs.iter().all(|x| self.has_type(x, t)),
            (Val::Array(xs), Type::Tuple(ts)) => xs.len() == ts.len() && xs.iter().zip(ts).all(|(x,t)| self.has_type(x, t)),
            _ => false,
        }
    }

    fn call(&self, name: &str, args: &[Val]) -> Result<Val, EvalError> {
        let func = self.lookup_fn(name)?;
        if func.args.len() != args.len() {
            return Err(EvalError::WrongNumberOfArgs(name.to_owned(), func.args.len(), args.len()));
        }
        let mut env = HashMap::new();
        for ((name, typ_expr), value) in func.args.iter().zip(args) {
            match self.eval(typ_expr, &env) {
                Ok(Val::Type(t)) => {
                    if self.has_type(value, &t) {
                        env.insert(name.to_owned(), value.clone());
                    } else {
                        return Err(EvalError::ArgIsWrongType(name.to_owned(), t.clone(), value.clone()));
                    }
                },
                Ok(_) => return Err(EvalError::ArgTypeNotType(name.to_owned())),
                Err(e) => return Err(EvalError::ArgTypeEvalError(name.to_owned(), Box::new(e))),
            }
        }
        let ret_type = match self.eval(&func.ret, &env) {
            Ok(Val::Type(t)) => t,
            Ok(_) => return Err(EvalError::RetTypeNotType),
            Err(e) => return Err(EvalError::RetTypeEvalError(Box::new(e))),
        };

        let result = match name {
            "list" => Val::Type(Type::List(Box::new(env.get("t").unwrap().unwrap_type()?))),
            "vector" => Val::Type(Type::Vector(
                    Box::new(env.get("t").unwrap().unwrap_type()?),
                    env.get("n").unwrap().unwrap_usize()?)
            ),
            "tuple" => Val::Type(Type::Tuple(env.get("ts").unwrap().unwrap_array_of_types()?)),
            _ => self.eval(&func.body, &env)?
        };

        if self.has_type(&result, &ret_type) {
            Ok(result)
        } else {
            Err(EvalError::ResultIsWrongType)
        }
    }

    fn eval(&self, expr: &Expr, env: &HashMap<String, Val>) -> Result<Val, EvalError> {
        match expr {
            Expr::Int(n) => Ok(Val::Int(n.clone())),
            Expr::Var(x) => self.lookup_value(x, env),
            Expr::Call(f, args) => {
                let mut arg_vals = vec![];
                for arg in args {
                    arg_vals.push(self.eval(arg, env)?);
                }
                self.call(f, &arg_vals)
            }
        }
    }

    pub fn eval_main(&self, args: &[String]) -> Result<Val, EvalError> {
        let args_val = Val::Array(args.iter().map(|s|Val::String(s.clone())).collect());
        self.call("main", &[args_val])
    }
}

impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for EvalError {}
