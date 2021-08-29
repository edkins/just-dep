mod ast;
mod combine;
mod eval;
mod parse;
mod typecheck;

use clap::{App, AppSettings, Arg};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new("justdep")
        .settings(&[AppSettings::TrailingVarArg])
        .arg(Arg::with_name("SCRIPT").required(true).help("Input script"))
        .arg(Arg::with_name("ARGS").multiple(true).help("Args to run script with"))
        .get_matches();

    let args:Vec<_> = if let Some(vs) = matches.values_of("ARGS") {
        vs.map(|s|s.to_owned()).collect()
    } else {
        vec![]
    };

    let input_filename = matches.value_of("SCRIPT").unwrap();

    let prelude_script = parse::parse(include_str!("prelude.jd"))?;

    let input = fs::read_to_string(input_filename)?;
    let script = parse::parse(&input)?;
    let program = combine::combine(&prelude_script, &script)?;
    typecheck::type_check(&program)?;
    let result = program.eval_main(&args)?;

    println!("{:?}", result);
    Ok(())
}
