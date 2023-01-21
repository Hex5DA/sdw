use std::fs;

mod lex;

fn main() {
    let contents = fs::read_to_string("examples/first.sdw").unwrap(); 
    let lexemes = lex::lex(contents);
    println!("Lexemes recieved: {:#?}", lexemes);
}

