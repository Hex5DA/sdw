#!/bin/bash
# hopefully i will go back to zsh at some point :)
# #!/bin/zsh

asm() {
    llc $1 -o ./a.s
    gcc ./a.s -o ./a
    ./a
    exitcode=$?
    rm ./a.s ./a
    return $exitcode
}

bc() {
    llvm-as $1 -o ./a.bc
    lli ./a.bc
    exitcode=$?
    rm ./a.ll ./a.bc
    return $exitcode
}

if [[ "$1" == "asm" ]]; then
    eval "asm $2"
    echo $?
fi

if [[ "$1" == "bc" ]]; then
    eval "bc $2"
    echo $?
fi

if [[ "$3" == "--clean" ]]; then
    rm $2
fi

if [[ "$#" -le 1 ]]; then
    echo "Syntax: run.sh [asm | bc] [path] --clean"
    echo "Options:"
    echo "  asm   -> compile the LLVM IR to assembly and then execute it"
    echo "  bc    -> compile the LLVM IR to LLVM bitcode and interpret it"
    echo "  clean -> remove the provided LLVM IR file"
fi

