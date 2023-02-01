# Shadow

A toy language I'm making for funsies!

Will sort this out properly, but some working design notes:

```
fn <ret type> <name>([<arg type> <arg>],?*) {
    <statement>*
}
```

Entry point is `fn int main()`.

Variable declration syntax:

```
<mod>* var <name> [as <type>]? = <expr>;
```

If the type is specified, `<expr>` should be coerced into `<type>`.

Modifiers:

- `mut` -> Mutable
- `dyn` -> Dynamic

Variables:

Variables have lifetimes, lasting from creation to deletion.

*Dynamic* variables, declared with the *dyn* keyword, can have 'multiple' lifetimes.
Ie. only one lifetime at a time, but this lifetime may change, and the parser / semantic
analyser will determine that this, and give it 'multiple' lifetimes.

Lifetimes declare a variable's type, name and modifiers.

Variables are immutable by default, & their value can't be changed.
Mutable variables may be declared with the *mut* keyword.
Their value may change but not their lifetime.

## Roadmap

- More tests for variables
- Build up the symbol table
- Improve the way variable expressions are handled, and allow for nested variables
- AST pretty printing (export to flowchart? )
- Operations on variables - start with mathematical operations
- Expand primitive data types (char, string, bool, more int type, ect.. )
- Yet more refactoring and cleaning up (as always :sweat_smile: )

## Development Strategy

I am building this compiler following the methodology presenting in the paper
["An Incremental Approach to Compiler Construction"](http://scheme2006.cs.uchicago.edu/11-ghuloum.pdf)

The development cycle goes something like as follows:

1) Add a 'small' feature to your subset of your language
2) Write tests to cover this feature
3) Implement it in code
4) Verify the code passes the tests
5) Refactor the compiler (or, the bits you have modified)
6) Repeat

This process means at every point of compiler development, the compiler works, and builds succesful code.

Testing is covered later in this document.

## Compile structure

### Lexical Analysis

Converting the input stream of characters (file content) into a series of lexemes that can be parsed

### Structural Analysis

Taking a stream of lexemes and parsing them into an AST.
This also adds metadata to different constructs - like attributing a return type to a function, ect.

### Semantic Analysis

Taking the AST and checking / defining its meaning. 

### Optimisational Analysis

Taking the semantically-associated AST and turning performing DCE, constant folding ect to
do some rough-optimisation before handing over to LLVM.

### Translational Analysis

Translating the AST into [LLVM] IR

After this, the IR can be built to ASM [and compiled] or built to LLVM bitcode and interpreted.

## Executing

TODO: Add notes on how [optimise LLVM IR / BC](https://llvm.org/docs/CommandGuide/opt.html)

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

I wrote a quick & dirty shell script to do this for me,
but there are no promises this'll work for you :sweat_smile:

## Testing

I wrote a simple testing framework in the form of a python script - `tests/test.py`.

Tests are organised into blocks, which are folders residing in the `tests/` directory.
These contain individual tests which are also folder; containing a `test.sdw` file and an `expected.ll` file.
Each `.sdw` file is compiled to LLVM IR and compared against the given `expected.ll` file.
The rest is just pretty printing :)

No promises about portability; the script uses `diff` and some icky stuff to run `cargo`,
so there's no guarantees :P

I imagine it'd be pretty simple to modify, though

`test.py` should be invoked with a list of the blocks to run, or standalone to run all blocks.

