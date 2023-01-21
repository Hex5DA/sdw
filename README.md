# Shadow

A toy language I'm making for funsies!

Will sort this out properly, but some working design notes:

```
fn <ret type> <name>([<arg type> <arg>],?*) {
    <statement>*
}
```

Entry point is `fn int main()`.

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

