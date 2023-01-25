#!/bin/python3

# tests/ contains different phases, representing the compiler
# these contain multiple directories containing individual tests
# each directory contains a `run.sdw` file and an `expected.ll` file
# the program is given a phase to search through
# it loops over all directories and compiles the `run.sdw` file and compares contents
#   - maybe print diffs of discrepancies?
# pretty printing
# maybe checks return codes of compiling / running the .sdw file / the compiler?

import subprocess
import argparse
import filecmp
import sys
import os

def error(err):
    print("\n\n")
    print("-" * 50)
    print(f"Error: {err}")
    print("-" * 50)
    sys.exit(-1)

def findfile(root, files, ideal, ext):
    if ideal not in files:
        for file in files:
            if file.rsplit(".")[1] == ext:
                return os.path.join(root, ideal)
        error(f"no {ext} file found in {root}")
    return os.path.join(root, ideal)

def block(name):
    count = succesful = 0
    for root, dirs, files in os.walk(name):
        if root.endswith(name):
            continue

        absroot = os.path.abspath(root)
        tname = os.path.basename(absroot) 
        print(f"Running test '{tname}': ", end="")
        prog = findfile(absroot, files, "test.sdw", ".sdw") 

        test = os.path.join(absroot, prog)
        result = os.path.join(absroot, "result.ll")
        expected = findfile(absroot, files, "expected.ll", ".ll")
        run = subprocess.run(["cargo",  "run", test, result], cwd="../", capture_output=True) 
        # https://doc.rust-lang.org/cargo/commands/cargo-run.html#exit-status
        if run.returncode == 101:
            error(f"the compiler did not return succesfully; output:\n\n{run.stderr.decode('utf-8')}")
        
        diff = subprocess.run(["diff", "-w", result, expected], capture_output=True)
        os.remove(result)
        passed = diff.returncode == 0 
        print("passed!" if passed else "failed...")
        if not passed:
            print()
            print("-" * 50)
            print("Unsuccesful test's diff log:")
            print(diff.stdout.decode("utf-8"))
            print("-" * 50)
            print()
        succesful += passed
        count += 1

    print()
    print(f"Block '{name}' finished; ({succesful}/{count})")

def main():
    os.chdir(os.path.abspath(os.path.dirname(__file__)))
    parser = argparse.ArgumentParser(prog = "ShadowLangTester", description = "Run automated tests for my compiler")
    parser.add_argument("blocks", default=["*"], nargs="?", action="append", help="You may specify '*' to run all blocks")
    args = parser.parse_args()
    if args.blocks == ["*", ["*"]]:
        args.blocks = [dir for dir in os.listdir(".") if os.path.isdir(dir)]

    for name in args.blocks:
        print()
        print("-" * 50)
        print(f"RUNNING BLOCK '{name}'")
        print("-" * 50)
        print()
        block(name)

if __name__ == "__main__":
    main()

