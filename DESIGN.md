# shadow lang design

i figured having something more complete might make development a little more collected
so here is that, or something.

## general syntax

### comments

- `//` from appearance to end of line
- `/* */` inline or multiline
- no further restrictions - eg. `//!` could be idiomatically a doc-comment, but this is not primitive nor standard-enforced

## structures

### statements

there are very few statements.
statements must be followed by a semicolon (;)

function declarations:
[TODO: re-reevaluate the feasiility of continuations in some form]

```sdw
fn [name]([[type] [name]]?*) [expr]
```

loops:
1 of 3 forms of control flow in `sdw`
the value of the `expr` is not utilised.

```sdw
loop [expr]
```

goto / labels:

```sdw
:[name]; // declare label
goto :[name]; // jump to a label
```

no `break` / `continue` primtives - encourage usage of `goto` for this purpose.

[TODO: consider ditching `loop` and just using `goto` + conditionals. could use convenience macro - like for `while` / `for`]

### expressions

the main structure of sdw
an expression evaluates to a singluar value.
this value can be discarded by following the offending expression with a semicolon (`;`)
*discarded values are still evaluated*

```sdw
fn foo() {
    std.printLn("side effect!");
}

fn main() {
    // although this expression is discarded, the print still runs.
    foo();
}
```

an unused expression value is a compiletime error.

### arithmetic

binary ops: `+`, `-`, `*`, `/`
bitwise ops: `|`, `&`, `~`, `^` `>>`, `<<`
logical operators: `&&`, `||`

as you'd expect

### blocks

a block - written `{ [constituant expressions]?* }` - evaluates to the value of the last member expressions
*this also applies to discarded final member expressions.* ie.

```sdw
{
    std.printLn("hello world!");
} // this block has a discarded value.
```

---
relatedly, expressions can be grouped just as one would expect with parenthese (`()`)

### conditional control flow

`if`, `else if` and `else` all work as you would expect.

```sdw
if [E^1^] [E^2^] [else if [E^1^] [E^2^]]?* [else [E^2^]]?
```

where:
`[E^1^]` -> evaluates to a boolean. the conditional.. conditional
`[E^2^]` -> an expression as the body. the matching arm is what the whole expression evaluates to.

