use std::fs::read_to_string;
use std::process;

use clap::Parser;

use sdw_lib::lex::lex;

#[derive(Parser)]
struct Args {
    filename: String,
}

fn main() {
    let args = Args::parse();
    let contents = read_to_string(&args.filename).unwrap_or_else(|err| {
        eprintln!(
            "could not read from the given file {}; error returned was\n{}",
            &args.filename, err
        );
        process::exit(1);
    });

    let lexemes = lex(&contents).unwrap_or_else(|err| {
        eprintln!("{}", err);
        err.verbose(&contents);
        process::exit(1);
    });

    // println!("[ DBG ] lexemes recieved;\n{:#?}", lexemes);
    let ast = sdw_lib::parse::parse(lexemes.into()).unwrap_or_else(|err| {
        eprintln!("{}", err);
        err.verbose(&contents);
        process::exit(1);
    });
    // println!("[ DBG ] AST built & recieved;\n{:#?}", ast);
}
