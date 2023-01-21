use std::fs;

mod parse;
mod lex;

fn main() {
    let contents = fs::read_to_string("examples/params.sdw").unwrap(); 
    let lexemes = lex::lex(contents);
    println!("Lexemes recieved: {:#?}", lexemes);
    parse::parse(lexemes);
}

