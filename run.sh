#!/bin/zsh

asm() {
    cargo run -- $1 ./a.ll
    llc ./a.ll -o ./a.s
    gcc ./a.s -o ./a
    ./a
    rm ./a.ll ./a.s ./a
}

bc() {
    cargo run -- $1 ./a.ll
    llvm-as ./a.ll -o ./a.bc
    lli ./a.bc
    rm ./a.ll ./a.bc
}

if [[ "$3" == "--quiet" ]]; then
    ext=" > /dev/null"
else
    ext=""
fi

if [[ "$1" == "asm" ]]; then
    eval "asm $2 $ext"
    echo $?
fi

if [[ "$1" == "bc" ]]; then
    eval "bc $2 $ext"
    echo $?
fi

if [[ "$#" -le 1 ]]; then
    echo "Syntax: run.sh [asm | bc] [path] --quiet?"
    echo "Options:"
    echo "  asm   -> SDWL will compile the LLVM IR to assembly and then execute it"
    echo "  bc    -> SDWL will compile the LLVM IR to LLVM bitcode and interpret it"
    echo "  quiet -> The output of the script will be hidden, however the exit code will still be printed"
fi

