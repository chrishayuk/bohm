#!/usr/bin/env python3
"""Check that everything the tutorial shows is what the tools really print.

A ```bohm block is a complete program. It must be followed by an ```output
block, optionally with an ```input block in between. The program is compiled
by Böhm's compiler, run on the machine, and its output compared; then it is
run again with its formulas read as algebra, which must print the same. A
program containing the line `# machine only` skips the second run.

A ```console block starts with a line `$ bohm ...`. The command is run from
the top folder of the repository and the rest of the block compared with
what it prints.

    python3 check_tutorial.py            check
    python3 check_tutorial.py --write    rewrite the expected text from real runs

The program is taken from $BOHM, or from ../target.
"""
import os
import pathlib
import re
import shlex
import subprocess
import sys
import tempfile

HERE = pathlib.Path(__file__).resolve().parent
ROOT = HERE.parent
BLOCK = re.compile(r"```(\w+)\n(.*?)```", re.S)


def binary():
    if os.environ.get("BOHM"):
        return os.environ["BOHM"]
    for build in ("release", "debug"):
        path = ROOT / "target" / build / "bohm"
        if path.exists():
            return str(path)
    sys.exit("no program found: run `cargo build` or set $BOHM")


def call(args, stdin=""):
    result = subprocess.run(args, input=stdin, capture_output=True, text=True,
                            timeout=60, cwd=ROOT)
    return (result.stdout + result.stderr).rstrip("\n")


def run_program(bohm, program, stdin):
    with tempfile.NamedTemporaryFile("w", suffix=".bohm", delete=False) as f:
        f.write(program)
    try:
        machine = call([bohm, "run", f.name], stdin).replace(f.name, "program.bohm")
        if "# machine only" in program:
            return machine, None
        algebra = call([bohm, "run", f.name, "--via", "interp"], stdin)
        algebra = algebra.replace(f.name, "program.bohm")
        return machine, None if algebra == machine else algebra
    finally:
        os.unlink(f.name)


def main():
    write = "--write" in sys.argv
    bohm = binary()
    checked = failed = 0
    for path in sorted(HERE.glob("*.md")):
        text = path.read_text()
        blocks = list(BLOCK.finditer(text))
        edits = []

        def compare(block, start, actual, what):
            nonlocal failed
            expected = text[start:block.end(2)].rstrip("\n")
            if actual == expected:
                return
            if write:
                edits.append((start, block.end(2), actual + "\n"))
            else:
                failed += 1
                print(f"{path.name}: {what}\n--- tutorial says\n{expected}\n"
                      f"--- the program prints\n{actual}\n")

        for i, block in enumerate(blocks):
            kind, body = block.group(1), block.group(2)
            if kind == "console":
                command, _, _ = body.partition("\n")
                if not command.startswith("$ bohm"):
                    continue
                checked += 1
                args = [bohm] + shlex.split(command[len("$ bohm"):])
                compare(block, block.start(2) + len(command) + 1, call(args), command)
            elif kind == "bohm":
                stdin, j = "", i + 1
                if j < len(blocks) and blocks[j].group(1) == "input":
                    stdin, j = blocks[j].group(2), j + 1
                if j >= len(blocks) or blocks[j].group(1) != "output":
                    print(f"{path.name}: program has no output block:\n{body}")
                    failed += 1
                    continue
                checked += 1
                machine, algebra = run_program(bohm, body, stdin)
                if algebra is not None:
                    failed += 1
                    print(f"{path.name}: the machine and the algebra disagree for:\n{body}"
                          f"--- machine\n{machine}\n--- algebra\n{algebra}\n")
                compare(blocks[j], blocks[j].start(2), machine, f"output differs for:\n{body}")
        for start, end, new in reversed(edits):
            text = text[:start] + new + text[end:]
        if edits:
            path.write_text(text)
            print(f"{path.name}: rewrote {len(edits)} block(s)")
    print(f"{checked} blocks checked, {failed} failed")
    sys.exit(1 if failed else 0)


if __name__ == "__main__":
    main()
