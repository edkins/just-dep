use crate::ast::{Expr, Func, Script};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::complete::{digit1, multispace0},
    combinator::{all_consuming, map, map_res, value},
    multi::many1,
    sequence::{delimited, preceded, terminated},
    Finish, IResult,
};
use std::collections::{HashMap, HashSet};
use std::{cmp::Ordering, fmt};

/**
 * Parsing entry point
 */
pub fn parse(input: &str) -> Result<Script, ParseErr> {
    Ok(all_consuming(preceded(whitespace, script))(input)
        .finish()
        .map_err(|e| ParseErr::new(e, input))?
        .1)
}

fn script(input: &str) -> IResult<&str, Script, Err> {
    map_res(many1(func), funcs_to_script)(input)
}

fn funcs_to_script(mut func_list: Vec<(String, Func)>) -> Result<Script, String> {
    let mut funcs = HashMap::new();
    for (name, func) in func_list.drain(..) {
        if funcs.contains_key(&name) {
            return Err(format!("Duplicate function: {}", name));
        }
        funcs.insert(name, func);
    }
    let declaration_order = func_list.iter().map(|x|x.0.clone()).collect();
    Ok(Script { declaration_order, funcs })
}

fn func(input: &str) -> IResult<&str, (String, Func), Err> {
    let (input, name) = word_owned(input)?;
    let (input, args) = many1(arg)(input)?;
    let arg_names: HashSet<_> = args.iter().map(|a| a.0.clone()).collect();
    if arg_names.len() < args.len() {
        return Err(nom::Err::Failure(Err {
            remaining: input.len(),
            message: "Duplicate argument name".to_owned(),
        }));
    }
    let (input, ()) = symbol(":")(input)?;
    let (input, ret) = expr(input)?;
    let (input, ()) = symbol("=")(input)?;
    let (input, body) = expr(input)?;
    let (input, ()) = symbol(";")(input)?;
    Ok((input, (name, Func { args, ret, body })))
}

fn arg(input: &str) -> IResult<&str, (String, Expr), Err> {
    let (input, ()) = symbol("(")(input)?;
    let (input, name) = word_owned(input)?;
    let (input, ()) = symbol(":")(input)?;
    let (input, typ) = expr(input)?;
    let (input, ()) = symbol(")")(input)?;
    Ok((input, (name, typ)))
}

fn expr(input: &str) -> IResult<&str, Expr, Err> {
    alt((word_with_args, tight_expr))(input)
}

fn tight_expr(input: &str) -> IResult<&str, Expr, Err> {
    alt((number, var, delimited(symbol("("), expr, symbol(")"))))(input)
}

fn word_with_args(input: &str) -> IResult<&str, Expr, Err> {
    let (input, name) = word_owned(input)?;
    let (input, params) = many1(tight_expr)(input)?;
    Ok((input, Expr::Call(name, params)))
}

fn var(input: &str) -> IResult<&str, Expr, Err> {
    map(word_owned, Expr::Var)(input)
}

fn number(input: &str) -> IResult<&str, Expr, Err> {
    map(terminated(digit1, whitespace), |s: &str| {
        Expr::Int(s.parse().unwrap())
    })(input)
}

///////////
//
// Combinators and stuff
//
///////////

fn symbol<'a, 'b: 'a>(sym: &'b str) -> impl Fn(&'a str) -> IResult<&'a str, (), Err> {
    move |input| {
        terminated(tagv(sym), whitespace)(input)
            .map_err(|e| decorate(e, format!("Expected: {:?}", sym)))
    }
}

/*
fn keyword<'a, 'b: 'a>(kw: &'b str) -> impl Fn(&'a str) -> IResult<&'a str, (), Err> {
    move |input| {
        let (input2, w) = word(input).map_err(|e| decorate(e, format!("Expected '{}'", kw)))?;
        if w == kw {
            Ok((input2, ()))
        } else {
            Err(nom::Err::Error(Err {
                remaining: input.len(), // "peek" semantics - error gives location at the start of the keyword
                message: format!("Expected '{}'", kw),
            }))
        }
    }
}
*/

fn tagv<'a, 'b: 'a>(t: &'b str) -> impl Fn(&'a str) -> IResult<&'a str, (), Err> {
    move |input| value((), tag(t))(input)
}

fn whitespace(input: &str) -> IResult<&str, (), Err> {
    value((), multispace0)(input)
}

fn word(input: &str) -> IResult<&str, &str, Err> {
    terminated(
        take_while1(|c: char| c.is_ascii_alphanumeric() || c == '_'),
        whitespace,
    )(input)
}

fn word_owned(input: &str) -> IResult<&str, String, Err> {
    map(word, str::to_owned)(input).map_err(|e| decorate(e, "word"))
}

//////////////
// My errors
//////////////

struct Err {
    remaining: usize,
    message: String,
}

#[derive(Debug)]
pub struct ParseErr {
    pub text: String,
    pub remaining: usize,
    pub message: String,
}

impl<'a> nom::error::ParseError<&'a str> for Err {
    fn from_error_kind(input: &'a str, kind: nom::error::ErrorKind) -> Self {
        Err {
            remaining: input.len(),
            message: format!("{:?}", kind),
        }
    }
    fn append(input: &'a str, kind: nom::error::ErrorKind, other: Self) -> Self {
        if other.remaining <= input.len() {
            other
        } else {
            Self::from_error_kind(input, kind)
        }
    }
    fn from_char(input: &'a str, x: char) -> Self {
        Err {
            remaining: input.len(),
            message: format!("Expected: {:?}", x),
        }
    }
    fn or(self, other: Self) -> Self {
        match other.remaining.cmp(&self.remaining) {
            Ordering::Equal => Err {
                remaining: self.remaining,
                message: format!("{} | {}", self.message, other.message),
            },
            Ordering::Less => other,
            Ordering::Greater => self,
        }
    }
}

impl Err {
    fn decorate(self, extra: impl fmt::Display) -> Self {
        Err {
            remaining: self.remaining,
            message: format!("{} {}", self.message, extra),
        }
    }
}

impl<'a> nom::error::FromExternalError<&'a str, String> for Err {
    fn from_external_error(input: &'a str, _kind: nom::error::ErrorKind, e: String) -> Self {
        Err {
            remaining: input.len(),
            message: e,
        }
    }
}

fn decorate(err: nom::Err<Err>, extra: impl fmt::Display) -> nom::Err<Err> {
    match err {
        nom::Err::Error(e) => nom::Err::Error(e.decorate(extra)),
        nom::Err::Failure(e) => nom::Err::Failure(e.decorate(extra)),
        e => e,
    }
}

impl ParseErr {
    fn new(e: Err, text: &str) -> Self {
        ParseErr {
            text: text.to_owned(),
            remaining: e.remaining,
            message: e.message,
        }
    }
}

impl fmt::Display for ParseErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let pos = self.text.len() - self.remaining;
        write!(
            f,
            "{}####{} {}",
            &self.text[..pos],
            &self.text[pos..],
            self.message
        )
    }
}

impl std::error::Error for ParseErr {}
