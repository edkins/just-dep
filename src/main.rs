mod ast;
mod eval;
mod parse;

use clap::{App, AppSettings, Arg};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new("justdep")
        .settings(&[AppSettings::TrailingVarArg])
        .arg(Arg::with_name("SCRIPT").required(true).help("Input script"))
        .arg(Arg::with_name("ARGS").multiple(true).help("Args to run script with"))
        .get_matches();

    let args:Vec<_> = matches.values_of("ARGS").unwrap().map(|s|s.to_owned()).collect();

    let input_filename = matches.value_of("SCRIPT").unwrap();
    let input = fs::read_to_string(input_filename)?;
    let script = parse::parse(&input)?;
    let result = script.eval_main(&args)?;

    println!("{:?}", result);
    Ok(())
}
