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
    print(f"error: {err}")
    sys.exit(-1)

def block(name):
    count = succesful = 0
    for root, dirs, files in os.walk(name):
        if root.endswith(name):
            continue

        absroot = os.path.abspath(root)
        tname = os.path.basename(absroot) 
        print(f"Running test '{tname}': ", end="")
        prog = "test.sdw"
        if "test.sdw" not in files:
            for file in files:
                if file.rsplit(".")[1] == "sdw":
                    prog = file
                    break
            error(f"no .sdw file found in {root}")

        test = os.path.join(absroot, prog)
        result = os.path.join(absroot, "result.ll")
        expected = os.path.join(absroot, "expected.ll")
        subprocess.run(["cargo",  "run", test, result], cwd="../", stdout = subprocess.DEVNULL, stderr = subprocess.STDOUT)
        os.remove(result)
        diff = subprocess.run(["diff", "--ignore-space", test, result], stdout = subprocess.DEVNULL, stderr = subprocess.STDOUT)
        passed = diff.returncode >= 0
        print("passed!" if passed else "failed...")
        if not passed:
            print("-" * 50)
            print("Unsuccesful test's diff log:")
            print(diff.stdout)
            print("-" * 50)
        succesful += passed
        count += 1

    print()
    print(f"Block '{name}' finished; ({succesful}/{count})")

def main():
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



