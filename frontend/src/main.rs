use clap::Parser;
use sdw::lex;
use std::fs;
use std::process;

#[derive(Parser, Debug)]
struct Args {
    /// filepath to read from
    input: String,
}

fn main() {
    let args = Args::parse();
    let contents = fs::read_to_string(args.input).unwrap_or_else(|_| {
        eprintln!(
            "error: could not read from input file '{}' - does it exist?",
            args.input
        );
        process::exit(1);
    });

    let tokens = lex::lex(contents).unwrap_or_else(|err| {
        eprintln!("TODO: error printing");
        eprintln!("{}", err);
        process::exit(1);
    });
}
