use std::collections::HashMap;
use std::fmt;
use num_bigint::BigInt;
use num_traits::cast::ToPrimitive;

use crate::ast::Expr;
use crate::combine::{Program, Func};

#[derive(Clone, Debug)]
pub enum Type {
    False,
    True,
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
    Int(BigInt),
    String(String),
    Array(Vec<Val>),
    Type(Type),
}

#[derive(Debug)]
pub enum EvalError {
    WrongNumberOfArgs(String, usize, usize),
    NoSuchFunc(String),
    NoSuchPreludeFunction(String),
    Overflow,
    NotInteger(Val),
    NotType(Val),
    NotArray(Val),
}

impl Val {
    fn unwrap_usize(&self) -> Result<usize, EvalError> {
        match self {
            Val::Int(i) => match i.to_usize() {
                None => Err(EvalError::Overflow),
                Some(n) => Ok(n),
            },
            _ => Err(EvalError::NotInteger(self.clone())),
        }
    }

    fn unwrap_type(&self) -> Result<Type, EvalError> {
        match self {
            Val::Type(t) => Ok(t.clone()),
            _ => Err(EvalError::NotType(self.clone())),
        }
    }

    fn unwrap_array(&self) -> Result<Vec<Val>, EvalError> {
        match self {
            Val::Array(xs) => Ok(xs.clone()),
            _ => Err(EvalError::NotArray(self.clone())),
        }
    }

    fn unwrap_array_of_types(&self) -> Result<Vec<Type>, EvalError> {
        self.unwrap_array()?.iter().map(Val::unwrap_type).collect()
    }
}

impl Program {
    fn lookup_or_compute_value(&self, name: &str, global_env: &mut HashMap<String, Val>, env: &HashMap<String, Val>) -> Result<Val, EvalError> {
        match env.get(name) {
            Some(x) => Ok(x.clone()),
            None => match global_env.get(name) {
                None => {
                    let x = self.call(name, &[], global_env)?;
                    global_env.insert(name.to_owned(), x.clone());
                    Ok(x)
                }
                Some(x) => Ok(x.clone()),
            }
        }
    }

    fn lookup_fn(&self, name: &str) -> Result<&Func, EvalError> {
        match self.funcs.get(name) {
            None => Err(EvalError::NoSuchFunc(name.to_owned())),
            Some(f) => Ok(f),
        }
    }

    /*
    fn has_type(&self, value: &Val, typ: &Type) -> bool {
        match (value, typ) {
            (Val::Type(Type::False), Type::Bool) | (Val::Type(Type::True), Type::Bool) | (Val::Int(_), Type::Int) | (Val::String(_), Type::String) | (Val::Type(_), Type::Type) => true,
            (Val::Int(n), Type::Uint) => n.sign() != Sign::Minus,
            (Val::Array(xs), Type::List(t)) => xs.iter().all(|x| self.has_type(x, t)),
            (Val::Array(xs), Type::Vector(t,n)) => xs.len() == *n && xs.iter().all(|x| self.has_type(x, t)),
            (Val::Array(xs), Type::Tuple(ts)) => xs.len() == ts.len() && xs.iter().zip(ts).all(|(x,t)| self.has_type(x, t)),
            _ => false,
        }
    }
    */

    fn call(&self, f: &str, args: &[Val], global_env: &mut HashMap<String, Val>) -> Result<Val, EvalError> {
        let func = self.lookup_fn(f)?;
        if func.args.len() != args.len() {
            return Err(EvalError::WrongNumberOfArgs(f.to_owned(), func.args.len(), args.len()));
        }

        let result = if func.prelude {
            match f {
                "true" => Val::Type(Type::True),
                "false" => Val::Type(Type::False),
                "bool" => Val::Type(Type::Bool),
                "int" => Val::Type(Type::Int),
                "uint" => Val::Type(Type::Uint),
                "string" => Val::Type(Type::String),
                "type" => Val::Type(Type::Type),
                "list" => Val::Type(Type::List(Box::new(args[0].unwrap_type()?))),
                "vector" => Val::Type(Type::Vector(
                        Box::new(args[0].unwrap_type()?),
                        args[1].unwrap_usize()?
                )),
                "tuple" => Val::Type(Type::Tuple(args[0].unwrap_array_of_types()?)),
                _ => return Err(EvalError::NoSuchPreludeFunction(f.to_owned())),
            }
        } else {
            let mut env = HashMap::new();
            for ((name, _), value) in func.args.iter().zip(args) {
                env.insert(name.clone(), value.clone());
            }

            self.eval(&func.body, global_env, &env)?
        };

        Ok(result)
    }

    fn eval(&self, expr: &Expr, global_env: &mut HashMap<String, Val>, env: &HashMap<String, Val>) -> Result<Val, EvalError> {
        match expr {
            Expr::Int(n) => Ok(Val::Int(n.clone())),
            Expr::Var(x) => self.lookup_or_compute_value(x, global_env, env),
            Expr::Call(f, args) => {
                let arg_vals:Vec<_> = args.iter().map(|x|self.eval(x,global_env,env)).collect::<Result<_,_>>()?;
                self.call(f, &arg_vals, global_env)
            }
            Expr::Array(xs) => {
                Ok(Val::Array(xs.iter().map(|x|self.eval(x,global_env,env)).collect::<Result<_,_>>()?))
            }
        }
    }

    pub fn eval_main(&self, args: &[String]) -> Result<Val, EvalError> {
        let args_val = Val::Array(args.iter().map(|s|Val::String(s.clone())).collect());
        let mut global_env = HashMap::new();
        self.call("main", &[args_val], &mut global_env)
    }
}

impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Eval error {:?}", self)
    }
}

impl std::error::Error for EvalError {}
