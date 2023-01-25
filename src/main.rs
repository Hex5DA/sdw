use clap::Parser;
use std::fs;
use std::process;

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
    let lexemes = lex::lex(contents).unwrap_or_else(|err| {
        eprintln!("An error occured whilst lexing the file:\n{}", err);
        process::exit(1);
    });

    println!("Lexemes recieved:\n{:#?}", lexemes);
    let ast = parse::parse(lexemes).unwrap();
    println!("AST built, and recieved:\n{:#?}", ast);
    println!("Generating IR..");
    ir::gen_ir(&mut ow, ast);
    ow.flush();
}
