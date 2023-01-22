use clap::Parser;
use std::fs;

mod lex;
mod parse;

#[derive(Parser)]
struct Args {
    filepath: String,
}

fn main() {
    let args = Args::parse();

    let contents = fs::read_to_string(args.filepath).unwrap();
    let lexemes = lex::lex(contents);
    println!("Lexemes recieved: {:#?}", lexemes);
    parse::parse(lexemes);
}
