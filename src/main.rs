mod ast;
mod parse;

use clap::clap_app;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = clap_app!(justdep =>
        (@arg INPUT: +required "input file")
    )
    .get_matches();

    let input_filename = matches.value_of("INPUT").unwrap();
    let input = fs::read_to_string(input_filename)?;
    let script = parse::parse(&input)?;

    println!("{:?}", script);
    Ok(())
}
