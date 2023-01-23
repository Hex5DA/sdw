use clap::Parser;
use std::fs;

mod ir;
mod lex;
mod parse;

#[derive(Parser)]
struct Args {
    filepath: String,
    #[arg(default_value = "a.ll")]
    ofile: String,
}

fn main() {
    let args = Args::parse();
    let mut ow = ir::OutputWrapper::new(args.ofile).unwrap();

    let contents = fs::read_to_string(args.filepath).unwrap();
    let lexemes = lex::lex(contents);
    println!("Lexemes recieved:\n{:#?}", lexemes);
    let ast = parse::parse(lexemes);
    println!("AST built, and recieved:\n{:#?}", ast);
    ir::gen_ir(&mut ow, ast);
    ow.flush();
}
