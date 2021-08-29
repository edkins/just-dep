use std::collections::HashMap;
use std::fmt;

use crate::ast::{Script, Expr};

#[derive(Debug)]
pub enum CombineError {
    DuplicateDecl(String),
    NoSuchDecl(String),
    Recursion(String),
}

#[derive(Clone, Debug)]
pub struct Program {
    pub order: Vec<String>,
    pub funcs: HashMap<String, Func>,
}

#[derive(Clone, Debug)]
pub struct Func {
    pub args: Vec<(String, Expr)>,
    pub ret: Expr,
    pub body: Expr,
    pub prelude: bool,
}

enum Visited {
    Visiting,
    Visited,
}

pub fn combine(prelude_script: &Script, main_script: &Script) -> Result<Program, CombineError> {
    let mut funcs = HashMap::new();
    for ((name, decl), prelude) in prelude_script.decls.iter().map(|d|(d,true)).chain(main_script.decls.iter().map(|d|(d,false))) {
        if funcs.contains_key(name) {
            return Err(CombineError::DuplicateDecl(name.clone()));
        }
        funcs.insert(name.clone(), Func {
            args: decl.args.clone(),
            ret: decl.ret.clone(),
            body: decl.body.clone(),
            prelude
        });
    }

    let prelude_order:Vec<_> = prelude_script.decls.iter().map(|d|d.0.clone()).collect();

    let mut visits = HashMap::new();
    for name in &prelude_order {
        visits.insert(name.clone(), Visited::Visited);
    }
    let mut program = Program {
        order: prelude_order,
        funcs,
    };
    for (name,_) in &main_script.decls {
        visit_for_ordering(&mut program, name, &mut visits)?;
    }
    Ok(program)
}

fn visit_for_ordering(program: &mut Program, name: &str, visits: &mut HashMap<String,Visited>) -> Result<(), CombineError> {
    if let Some(v) = visits.get(name) {
        match v {
            Visited::Visited => Ok(()),
            Visited::Visiting => Err(CombineError::Recursion(name.to_owned())),
        }
    } else {
        visits.insert(name.to_owned(), Visited::Visiting);
        for dep in &get_dependencies(program, name)? {
            visit_for_ordering(program, dep, visits)?;
        }
        visits.insert(name.to_owned(), Visited::Visited);
        program.order.push(name.to_owned());
        Ok(())
    }
}

fn get_dependencies(program: &Program, name: &str) -> Result<Vec<String>, CombineError> {
    if let Some(func) = program.funcs.get(name) {
        let mut result = vec![];
        for arg in &func.args {
            add_dependencies(&arg.1, &mut result);
        }
        add_dependencies(&func.ret, &mut result);
        add_dependencies(&func.body, &mut result);
        Ok(result)
    } else {
        Err(CombineError::NoSuchDecl(name.to_owned()))
    }
}

fn add_dependencies(expr: &Expr, result: &mut Vec<String>) {
    match expr {
        Expr::Int(_) => {}
        Expr::Var(x) => {
            if !result.contains(x) {
                result.push(x.clone());
            }
        }
        Expr::Call(f,xs) => {
            if !result.contains(f) {
                result.push(f.clone());
            }
            for x in xs {
                add_dependencies(x, result);
            }
        }
        Expr::Array(xs) => {
            for x in xs {
                add_dependencies(x, result);
            }
        }
    }
}

impl fmt::Display for CombineError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Combine error {:?}", self)
    }
}

impl std::error::Error for CombineError {}
