use clap::Parser;
use owo_colors::OwoColorize;
use sdw::lexer;
use std::fs;
use std::process;

#[derive(Parser, Debug)]
struct Args {
    /// filepath to read from
    input: String,
}

// TODO: extract to `errors.rs`?
/// "whilst ${process}"
fn print_errs(state: &sdw::common::State, contents: &String, process: &str) {
    let err_text = format!(
        "{} error{}",
        state.errors.len(),
        if state.errors.len() == 1 { "" } else { "s" }
    );

    eprintln!(
        "\nsummary: {} raised whilst {}.\n",
        err_text.red(),
        process.bright_green()
    );

    for (idx, error) in state.errors.iter().enumerate() {
        eprintln!("\n~= {} #{} =~", "error".red(), idx + 1);
        error.print(&contents);
    }

    process::exit(1);
}

fn main() {
    let args = Args::parse();
    let contents = fs::read_to_string(&args.input).unwrap_or_else(|_| {
        eprintln!(
            "error: could not read from input file '{}' - does it exist?",
            args.input
        );
        process::exit(1);
    });

    let mut state = sdw::common::State::new();
    let lexemes = lexer::lex(&mut state, &contents);
    if !state.errors.is_empty() {
        print_errs(&state, &contents, "lexing");
    }

    println!(
        "lexemes:\n{}",
        lexemes
            .iter()
            .map(|tk| format!("{:?}", tk.spanned))
            .collect::<Vec<String>>()
            .join(" ")
    );

    sdw::parser::parse(&mut state, lexemes);
    if !state.errors.is_empty() {
        print_errs(&state, &contents, "parsing");
    }
}
