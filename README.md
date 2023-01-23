# Shadow

A toy language I'm making for funsies!

Will sort this out properly, but some working design notes:

```
fn <ret type> <name>([<arg type> <arg>],?*) {
    <statement>*
}
```

Entry point is `fn int main()`.

## Roadmap

- Refactor of parsing (and maybe IR emitting)
- Develop / utilise a testing framework
- Variables

## Compile structure

### Lexical Analasis

Converting the input stream of characters (file content) into a series of lexemes that can be parsed

### Structural Analasis

Taking a stream of lexemes and parsing them into an AST.
This also adds metadata to different constructs - like attributing a return type to a function, ect.

### Semantic Analasis

Taking the AST and checking / defining its meaning. 

### Translational Analasis

Translating the AST into [LLVM] IR

After this, the IR can be built to ASM [and compiled] or built to LLVM bitcode and interpreted.

## Executing

1) Run the compiler with `cargo run <path> <opath>` (run `cargo run --help` for details)
2) Generate an assembly file with `llc <opath> -o <lpath>`
3) Assemble the assembly file with, eg. `gcc <lpath> -o <exec>`
4) Run the executable with `<exec>`
5) Check this exit code, if you want with `echo $?`

You could also build to LLVM bitcode with: 

1) Run the compiler with `cargo run <path> <opath>` (run `cargo run --help` for details)
2) Generate an LLVM bitcode file with `llvm-as <opath> -o <bpath>`
3) Interpet this bitcode file with `lli <bpath>`
4) Check the exit code, if you want with `echo $?`

I wrote a quick & shell script a quick shell script to do this for me,
but there are no promises this'll work for you :sweat_smile:

