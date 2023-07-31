use clap::Parser;
use owo_colors::OwoColorize;
use sdw::lexer;
use std::fs;
use std::process;
use std::time::Instant;

#[derive(Parser, Debug)]
struct Args {
    /// filepath to read from
    input: String,

    /// print extra information
    #[arg(short, long)]
    verbose: bool,
}

mod print {
    use std::time::Instant;

    use owo_colors::OwoColorize;
    use sdw::prelude::*;

    macro_rules! print_idn {
        ($ident:expr, $($arg:tt)+) => {{
            let whitespace = std::iter::repeat(' ').take($ident * 2).collect::<String>();
            println!("{}{}", whitespace, format_args!($($arg)+));
        }};
    }

    fn stn(node: &STN, ident: usize) {
        match &node.spanned {
            ST::Stmt(stmt) => match stmt {
                Stmt::Fn {
                    return_type,
                    name,
                    parameters,
                    body,
                } => {
                    print_idn!(ident, "function:");
                    print_idn!(ident + 1, "name -> {}", name.spanned);
                    print_idn!(ident + 1, "return type -> {}", return_type.spanned);
                    print_idn!(ident + 1, "parameters:");

                    for (r#type, name) in parameters {
                        print_idn!(
                            ident + 2,
                            "type, name -> {}, {}",
                            r#type.spanned,
                            name.spanned
                        );
                    }
                    if parameters.is_empty() {
                        print_idn!(ident + 2, "[ none ]");
                    }

                    print_idn!(ident + 1, "body:");
                    st(body, ident + 2);
                }
                Stmt::Stub {
                    return_type,
                    name,
                    parameters,
                } => {
                    print_idn!(ident, "function stub:");
                    print_idn!(ident + 1, "name -> {}", name.spanned);
                    print_idn!(ident + 1, "return type -> {}", return_type.spanned);
                    print_idn!(ident + 1, "parameters:");

                    for r#type in parameters {
                        print_idn!(ident + 2, "type -> {}", r#type.spanned);
                    }
                    if parameters.is_empty() {
                        print_idn!(ident + 2, "[ none ]");
                    }
                }
                Stmt::Loop { block } => {
                    print_idn!(ident, "loop:");
                    st(block, ident + 1);
                },
                Stmt::Label { name } => {
                    print_idn!(ident, "label:");
                    print_idn!(ident + 1, "name -> {}", name.spanned);
                },
                Stmt::Goto { name } => {
                    print_idn!(ident, "goto:");
                    print_idn!(ident + 1, "target -> {}", name.spanned);
                },
                Stmt::Return { expr } => {
                    print_idn!(ident, "return:");
                    if let Some(expr) = expr{
                        stn(&expr, ident + 1);
                    } else {
                        print_idn!(ident + 1, "[ no return expression ]");
                    }
                },
                Stmt::VarDec { name, initialiser } => {
                    print_idn!(ident, "variable declaration:");
                    print_idn!(ident, "name -> {}", name.spanned);
                    stn(&initialiser, ident + 1);
                },
                Stmt::VarRes { name, updated } => {
                    print_idn!(ident, "variable reassignment:");
                    print_idn!(ident, "name -> {}", name.spanned);
                    stn(&updated, ident + 1);
                },
                Stmt::Type { name, bound } => {
                    print_idn!(ident, "type declaration");
                    print_idn!(ident + 1, "name -> {}", name.spanned);
                    stn(bound, ident + 1);
                }
            },
            other => println!("{:#?}", other),
        }
    }

    /// `ident` here is the identation for the block to be prefixed with
    /// (ie. it is the _caller's_ responsibility to `ident + 1`)
    fn st(root: &Vec<STN>, ident: usize) {
        for node in root {
            stn(node, ident)
        }
    }

    pub fn syntax_tree(root: &Vec<STN>) {
        st(root, 0);
    }

    pub fn lexemes(lexemes: &Vec<Lexeme>) {
        println!(
            "lexemes:\n{}",
            lexemes
                .iter()
                .map(|tk| format!("{:?}", tk.spanned))
                .collect::<Vec<String>>()
                .join(" ")
        );
    }

    pub fn done(before: &Instant) {
        println!(
            "{}, in {} μs",
            "done".to_owned().bright_green(),
            before.elapsed().as_micros().bright_green(),
        );
    }
}

fn main() {
    let args = Args::parse();
    let contents = fs::read_to_string(&args.input).unwrap_or_else(|_| {
        eprintln!(
            "{}: could not read from input file '{}' - does it exist?",
            "error".red(),
            args.input
        );
        process::exit(1);
    });

    let mut state = sdw::common::State::new();

    let before = Instant::now();
    println!("{}..", "lexing file".bright_green());
    let lexemes = lexer::lex(&mut state, &contents);

    if !state.errors.is_empty() {
        state.print_errs(&contents, "lexing");
        process::exit(1);
    }

    print::done(&before);
    println!(
        "produced {} lexemes",
        lexemes.len().bright_green()
    );

    println!();
    if args.verbose {
        print::lexemes(&lexemes);
        println!();
    }

    let before = Instant::now();
    println!("{}..", "parsing file".bright_green());
    let st = sdw::parser::parse(&mut state, lexemes).unwrap_or_else(|err| {
        #[rustfmt::skip]
        let err_text = format!( // i don't know how better to write this. deal with it. it lines up
            r"
              an {} was raised: 
            ======================================
            ",
            "unrecoverable error".red()
        );

        eprintln!("{}", err_text);
        err.print(&contents);
        process::exit(1);
    });

    print::done(&before);
    if !state.errors.is_empty() {
        state.print_errs(&contents, "parsing");
        process::exit(1);
    }

    println!();
    if args.verbose {
        print::syntax_tree(&st);
        println!();
    }
}
