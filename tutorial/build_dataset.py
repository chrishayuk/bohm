#!/usr/bin/env python3
"""Build dataset/bohm.jsonl, a chat-format training set about Böhm's language.

Every program in an answer is compiled by Böhm's compiler and run on the
machine, and the output shown in the answer is what it actually printed, so
the dataset cannot drift from the language. Run from this folder after
building the tools:

    python3 build_dataset.py
"""
import json
import os
import pathlib
import subprocess
import sys
import tempfile

from check_tutorial import binary

HERE = pathlib.Path(__file__).resolve().parent
EXAMPLES = HERE.parent / "examples"
SYSTEM = ("You are a knowledgeable assistant that provides detailed information "
          "about the programming language of Corrado Böhm's 1951 dissertation.")
BOHM = binary()
rows = []


def run(src, stdin=""):
    with tempfile.NamedTemporaryFile("w", suffix=".bohm", delete=False) as f:
        f.write(src + "\n")
    try:
        result = subprocess.run([BOHM, "run", f.name], input=stdin, capture_output=True,
                                text=True, timeout=60)
    finally:
        os.unlink(f.name)
    if result.returncode != 0:
        sys.exit(f"program failed:\n{src}\n{result.stderr}")
    return result.stdout.rstrip("\n")


def prog(src, stdin=""):
    """A program, its input if any, and its real output, as markdown."""
    src = src.strip("\n")
    text = f"```bohm\n{src}\n```\n\n"
    if stdin:
        text += f"With the input `{stdin.strip()}` it prints:\n\n"
    else:
        text += "It prints:\n\n"
    return text + f"```\n{run(src, stdin)}\n```"


def qa(question, answer):
    rows.append({"messages": [
        {"role": "system", "content": SYSTEM},
        {"role": "user", "content": question},
        {"role": "assistant", "content": answer.strip()},
    ]})


# ---------------------------------------------------------------- concepts

qa("What is Böhm's programming language?",
   "It is the language of Corrado Böhm's doctoral dissertation, submitted at ETH "
   "Zürich in 1951 and printed in 1954. The dissertation describes a computer, a "
   "language of formulas for programming it, and a compiler for that language "
   "written in the language itself. Every statement is an assignment, written "
   "`expression → variable`. Numbers are whole and never negative, and names are "
   "single letters. Here is a complete program:\n\n"
   + prog("π' → A\n? → a\n? → b\na + b → ?\nΩ → π", "3 4"))

qa("Does Böhm's language have a name?",
   "No. Böhm did not name it; the dissertation only describes its formulas. It is "
   "usually called Böhm's language. The dissertation is \"Calculatrices digitales: "
   "du déchiffrage de formules logico-mathématiques par la machine même dans la "
   "conception du programme\".")

qa("How do I print a number in Böhm's language?",
   "Assign it to `?`. On the right of the arrow, `?` is the output.\n\n"
   + prog("π' → A\n1951 → ?\nΩ → π"))

qa("How do I read input in Böhm's language?",
   "Assign from `?`. On the left of the arrow, `?` is the next number of the "
   "input.\n\n" + prog("π' → A\n? → n\nn·2 → ?\nΩ → π", "21"))

qa("How do I stop a program in Böhm's language?",
   "Write `Ω → π`. `π` holds the place the program continues from, and `Ω` is no "
   "place at all. A program also stops when it runs past its last formula.\n\n"
   + prog("π' → A\n1 → ?\nΩ → π\n2 → ?"))

qa("What does π' → A mean in Böhm's language?",
   "It names a group. `π′` is the place where the next formula will be, so "
   "`π' → A` makes A the name of the group of formulas that begins there. Other "
   "formulas can then continue at A by writing `A → π`. A program starts at its "
   "first group.")

qa("How do I jump to another part of a program in Böhm's language?",
   "Assign a group to `π`. There is no jump statement; `π` is the variable that "
   "holds where to continue, and a jump is an assignment to it.\n\n"
   + prog("π' → A\n1 → ?\nC → π\n\nπ' → B\n2 → ?\nΩ → π\n\nπ' → C\n3 → ?\nB → π"))

qa("What are the operations in Böhm's language?",
   "There are eight: `+` sum, `·` product, `∸` difference that stops at 0, `:` "
   "quotient, `÷` absolute difference, `mod` remainder, `∪` the greater of two "
   "numbers and `∩` the smaller.\n\n"
   + prog("π' → A\n? → a\n? → b\na + b → ?\na·b → ?\na ∸ b → ?\na:b → ?\na ÷ b → ?\n"
          "a mod b → ?\na ∪ b → ?\na ∩ b → ?\nΩ → π", "17 5"))

qa("Can I type Böhm's language on an ordinary keyboard?",
   "Yes, in this implementation. `->` is `→`, `pi` is `π`, `pi'` is `π′`, `Omega` "
   "is `Ω`, `@x` is `↓x`, `*` is `·`, `-` is `∸`, `/` is `:`, `~` is `÷`, `max` is "
   "`∪` and `min` is `∩`.\n\n"
   + prog("pi' -> A\n? -> a\n? -> b\na*b - 1 -> ?\nOmega -> pi", "6 7"))

qa("Are there negative numbers in Böhm's language?",
   "No. Numbers are whole and never negative. The subtraction `∸` stops at 0, so "
   "`3 ∸ 5` is 0. `÷` gives the size of a difference whichever number is the "
   "larger.\n\n" + prog("π' → A\n3 ∸ 5 → ?\n3 ÷ 5 → ?\nΩ → π"))

qa("How large can a number be in Böhm's language?",
   "A cell holds fourteen decimal digits, so the largest number is "
   "99 999 999 999 999. A sum or product that is too large keeps its last "
   "fourteen digits.\n\n" + prog("π' → A\n99999999999999 + 2 → ?\nΩ → π"))

qa("What happens when I divide by zero in Böhm's language?",
   "The machine stops. `:` and `mod` with a second operand of 0 are not defined, "
   "and this implementation reports `division by zero`. Test the divisor first if "
   "it can be 0.")

qa("How do I write an if statement in Böhm's language?",
   "There is none. A program chooses by computing which group to continue at. "
   "`1 ∩ x` is 1 when x is not 0, and `1 ∸ x` is 1 when x is 0. Multiply each "
   "group by one of them and add; the group multiplied by 1 is where the program "
   "continues.\n\n"
   + prog("π' → A\n? → n\n(1 ∩ n)·P + (1 ∸ n)·Z → π\n\nπ' → P\n1 → ?\nΩ → π\n\n"
          "π' → Z\n0 → ?\nΩ → π", "7"))

qa("How do I compare two numbers in Böhm's language?",
   "With subtraction that stops at 0. `1 ∩ (x ∸ y)` is 1 when x is greater than "
   "y, `1 ∸ (x ∸ y)` is 1 when x is at most y, `1 ∸ (x ÷ y)` is 1 when they are "
   "equal and `1 ∩ (x ÷ y)` is 1 when they differ.\n\n"
   + prog("π' → A\n? → x\n? → y\n1 ∩ (x ∸ y) → ?\n1 ∸ (x ∸ y) → ?\n1 ∸ (x ÷ y) → ?\n"
          "Ω → π", "9 4"))

qa("How do I write 'and', 'or' and 'not' in Böhm's language?",
   "Conditions are numbers that are 0 or 1, so they are arithmetic. p and q is "
   "`p·q`. p or q is `1 ∩ (p + q)`. Not p is `1 ∸ p`.\n\n"
   + prog("π' → A\n? → n\n(1 ∸ (10 ∸ n))·(1 ∸ (n ∸ 20)) → ?\nΩ → π", "14")
   + "\n\nThe formula is 1 when n is at least 10 and at most 20.")

qa("Why does my program in Böhm's language stop with 'π = N is outside the memory'?",
   "A choice gave `π` something that is not one group. The usual cause is two "
   "multipliers that are 1 together, so that `π` receives the sum of two groups. "
   "In `(1 ∩ c)·K + (1 ∸ c)·L → π` exactly one multiplier is 1 for every c; check "
   "that yours have the same property. Böhm made this mistake in group C of his "
   "own compiler.")

qa("How do I write a loop in Böhm's language?",
   "A loop is a group that continues at itself. Put the test in a group of its "
   "own: it continues at the body while the condition holds, and otherwise "
   "somewhere else or nowhere.\n\n"
   + prog("π' → A\n? → n\n\nπ' → B\n(1 ∩ n)·C → π\n\nπ' → C\nn → ?\nn ∸ 1 → n\nB → π",
          "5"))

qa("How do I count from 1 to n in Böhm's language?",
   "Loop while i is at most n. `1 ∸ (i ∸ n)` is 1 exactly then.\n\n"
   + prog("π' → A\n? → n\n1 → i\n\nπ' → B\n(1 ∸ (i ∸ n))·C → π\n\nπ' → C\ni → ?\n"
          "i + 1 → i\nB → π", "5"))

qa("Why does my loop never stop in Böhm's language?",
   "The test must eventually give `π` a group outside the loop, or 0. The usual "
   "causes are a missing formula to change the loop variable, and a test such as "
   "`1 ∩ (n ∸ 1)` that reaches 0 one step early or late. A loop with no way out "
   "is stopped by this implementation after 50 million steps with `step limit "
   "reached`.")

qa("Does Böhm's language have arrays?",
   "It has indirection. `↓x` is the cell whose address is in x, so a list is a "
   "run of cells and an index is an address. Cells 100 to 499 are free for lists "
   "in this implementation.\n\n"
   + prog("π' → A\n100 → i\n\nπ' → B\n(1 ∩ (103 ∸ i))·C + (1 ∸ (103 ∸ i))·D → π\n\n"
          "π' → C\n? → ↓i\ni + 1 → i\nB → π\n\nπ' → D\n(1 ∩ (i ∸ 100))·E → π\n\n"
          "π' → E\ni ∸ 1 → i\n↓i → ?\nD → π", "10 20 30"))

qa("What does ↓ mean in Böhm's language?",
   "`↓x` is the cell whose address is the value of x. `a → ↓x` stores a there, "
   "and `↓x → a` fetches from there. Only a variable can follow `↓`; to use a "
   "computed address, put it in a variable first.\n\n"
   + prog("π' → A\n200 → p\n7 → ↓p\n↓p + 1 → ?\nΩ → π"))

qa("How do I write a subroutine in Böhm's language?",
   "Böhm does not describe subroutines, but a group name is a number that can be "
   "stored. The caller puts the group to come back to in an agreed variable and "
   "jumps; the subroutine ends by assigning that variable to `π`.\n\n"
   + prog("π' → A\n6 → a\nB → z\nD → π\n\nπ' → B\nr → ?\n20 → a\nC → z\nD → π\n\n"
          "π' → C\nr → ?\nΩ → π\n\nπ' → D\na·2 → r\nz → π"))

qa("Does Böhm's language support recursion?",
   "Not directly: variables are shared by the whole program and there is no call "
   "stack. A program can keep its own stack in memory with `↓`, saving what each "
   "call must remember. `examples/hanoi.bohm` and `examples/ackermann.bohm` do "
   "this.")

qa("Why must I not name a variable S or X in Böhm's language?",
   "Because of how formulas are compiled. A formula without brackets around a "
   "sum is compiled by Böhm's polynomial method, which keeps the running sum in "
   "the variable S and the running product in X. A program that uses S or X for "
   "its own purposes and also contains such a formula has them overwritten. Read "
   "as algebra the program is correct, and on the machine it is not. Knuth and "
   "Trabb Pardo's 1976 version of the TPK algorithm in Böhm's language uses both "
   "letters.")

qa("What is the order of operations in Böhm's language?",
   "`·` and `:` are done before `+` and `∸`, and operations of equal rank from "
   "left to right. `÷`, `mod`, `∪` and `∩` must stand alone in a formula or be "
   "put in brackets. `(`, `[` and `{` are interchangeable.\n\n"
   + prog("π' → A\n2 + 3·4 → ?\n(2 + 3)·4 → ?\n20:2:5 → ?\n(17 mod 5) + 1 → ?\nΩ → π"))

qa("How do I write a comment in Böhm's language?",
   "The dissertation has none; Böhm printed explanations beside the formulas. In "
   "this implementation `#` begins a comment that runs to the end of the line.\n\n"
   + prog("# Print one number.\nπ' → A\n42 → ?            # the answer\nΩ → π"))

qa("How do I find a remainder in Böhm's language?",
   "`mod` is one of the eight operations. Put it in brackets when it is part of "
   "a larger formula.\n\n" + prog("π' → A\n17 mod 5 → ?\n(17 mod 5)·10 → ?\nΩ → π"))

qa("How do I test whether a number is even in Böhm's language?",
   "`n mod 2` is 1 for an odd number and 0 for an even one, so it can be used "
   "directly as the multiplier in a choice.\n\n"
   + prog("π' → A\n? → n\n(n mod 2)·P + (1 ∸ (n mod 2))·Q → π\n\nπ' → P\n1 → ?\n"
          "Ω → π\n\nπ' → Q\n2 → ?\nΩ → π", "10"))

qa("How do I find the larger of two numbers in Böhm's language?",
   "`∪` is the greater of two numbers and `∩` the smaller.\n\n"
   + prog("π' → A\n? → a\n? → b\na ∪ b → ?\na ∩ b → ?\nΩ → π", "12 31"))

qa("How do I swap two variables in Böhm's language?",
   "Use a third variable, or add and subtract:\n\n"
   + prog("π' → A\n? → a\n? → b\na + b → a\na ∸ b → b\na ∸ b → a\na → ?\nb → ?\nΩ → π",
          "17 42"))

qa("How do I round a division up in Böhm's language?",
   "`:` rounds down. Add one less than the divisor first to round up, or half "
   "the divisor to round to the nearest.\n\n"
   + prog("π' → A\n? → a\n? → b\na:b → ?\n(a + b ∸ 1):b → ?\n(a + b:2):b → ?\nΩ → π",
          "17 5"))

qa("What machine does Böhm's language run on?",
   "A computer Böhm designed in the first chapter of the dissertation, which was "
   "never built. It has about 1000 cells of fourteen decimal digits. Cell 000 is "
   "`π`, the address of the next instruction, and cell 001 is the input and "
   "output. An instruction is itself a fourteen-digit number: two operand "
   "addresses, an operation code and a result address. Every instruction "
   "computes one result from two operands and stores it; a jump is an "
   "instruction that stores into cell 000.")

qa("How did Böhm's compiler work?",
   "It is a program of 31 groups in chapter 7 of the dissertation, written in "
   "the language it compiles. It reads a formula as a sequence of numbers from a "
   "teletypewriter, one per symbol, and writes instruction numbers. For a fully "
   "bracketed formula it keeps one cell for each open parenthesis, in which it "
   "builds the instruction for that parenthesis, and it writes the instructions "
   "for the innermost parentheses first.")

qa("Was Böhm's compiler correct as printed?",
   "Not as printed in 1954. Run on a simulation of his machine, it needs four "
   "corrections. Group C sends `π` to the sum of two groups for most symbols. "
   "Group E′ leaves the operation code out of a plain store. Group I overwrites "
   "the link to the enclosing parenthesis, a bug Knuth and Trabb Pardo reported "
   "in 1976. And the compiler uses the variable X, which the code generated for "
   "its own formulas also uses. With all four corrected it compiles its own "
   "text, and the result compiles that text again to the same numbers.")

qa("Who was Corrado Böhm?",
   "An Italian computer scientist (1923–2017). He wrote his dissertation at ETH "
   "Zürich under Eduard Stiefel and Paul Bernays, submitted in 1951. He is also "
   "known for the Böhm–Jacopini theorem of 1966 on structured programs, for the "
   "language P′′, and for Böhm's theorem in the lambda calculus.")

qa("What are the three readings of a program in this implementation of Böhm's language?",
   "A program can be run three ways. Its formulas can be read as algebra, with "
   "no machine. It can be translated to instruction numbers by a second "
   "compiler written in Rust from chapters 5 and 6 of the dissertation. Or it "
   "can be translated by Böhm's own compiler from chapter 7, itself running on "
   "the simulated machine. `bohm verify FILE` does all three and compares the "
   "results.")

qa("Can a program in Böhm's language change itself?",
   "On the machine, yes. Instructions are numbers in the same memory as "
   "everything else, and a group name is the address of its first instruction, "
   "so a formula can store a new instruction number there. Read as algebra such "
   "a program has no meaning. `examples/self-modifying.bohm` is an example.")

# ---------------------------------------------------------------- programs

TASKS = {
    "echo": "Write a program in Böhm's language that reads a number and prints it.",
    "add": "Write a program in Böhm's language that adds two numbers.",
    "arithmetic": "Show me examples of arithmetic in Böhm's language.",
    "compare": "Write a program in Böhm's language that compares two numbers.",
    "countdown": "Write a program in Böhm's language that counts down from a number.",
    "even-odd": "Write a program in Böhm's language that says whether numbers are even or odd.",
    "max": "Write a program in Böhm's language that finds the greatest and least of three numbers.",
    "swap": "Write a program in Böhm's language that exchanges two numbers.",
    "sum-to-n": "Write a program in Böhm's language that adds up the numbers from 1 to n.",
    "multiplication-table": "Write a program in Böhm's language that prints a multiplication table.",
    "triangle-numbers": "Write a program in Böhm's language that prints triangular numbers.",
    "temperature": "Write a program in Böhm's language that prints a Fahrenheit to Celsius table.",
    "fibonacci": "Write a program in Böhm's language that prints Fibonacci numbers.",
    "factorial": "Write a program in Böhm's language that calculates factorials.",
    "power": "Write a program in Böhm's language that raises a number to a power.",
    "gcd-lcm": "Write a program in Böhm's language that finds the greatest common divisor and least common multiple.",
    "collatz": "Write a program in Böhm's language that prints a Collatz sequence.",
    "primes": "Write a program in Böhm's language that prints prime numbers.",
    "prime-factors": "Write a program in Böhm's language that prints the prime factors of a number.",
    "perfect-numbers": "Write a program in Böhm's language that finds perfect numbers.",
    "digits": "Write a program in Böhm's language that works with the digits of a number.",
    "palindromes": "Write a program in Böhm's language that finds numbers that read the same backwards.",
    "binary": "Write a program in Böhm's language that converts numbers to binary.",
    "square-root": "Write a program in Böhm's language that calculates an integer square root.",
    "leap-years": "Write a program in Böhm's language that tests for leap years.",
    "day-of-week": "Write a program in Böhm's language that finds the day of the week of a date.",
    "change": "Write a program in Böhm's language that makes change with the fewest coins.",
    "horner": "Write a program in Böhm's language that evaluates a polynomial.",
    "random": "Write a program in Böhm's language that prints pseudo-random dice rolls.",
    "statistics": "Write a program in Böhm's language that prints statistics for a list of numbers.",
    "array-reverse": "Write a program in Böhm's language that reads numbers into a list and prints them in reverse.",
    "bubble-sort": "Write a bubble sort in Böhm's language.",
    "insertion-sort": "Write an insertion sort in Böhm's language.",
    "binary-search": "Write a binary search in Böhm's language.",
    "sieve": "Write the sieve of Eratosthenes in Böhm's language.",
    "pascal-triangle": "Write a program in Böhm's language that prints Pascal's triangle.",
    "josephus": "Write a program in Böhm's language that solves the Josephus problem.",
    "subroutine": "Show me a subroutine in Böhm's language.",
    "jump-table": "Write a calculator in Böhm's language that uses a table of jumps.",
    "hanoi": "Write the Towers of Hanoi in Böhm's language.",
    "ackermann": "Write Ackermann's function in Böhm's language.",
    "self-modifying": "Write a program in Böhm's language that changes one of its own instructions.",
}
missing = {p.stem for p in EXAMPLES.glob("*.bohm")} ^ set(TASKS)
if missing:
    sys.exit(f"examples and TASKS differ: {sorted(missing)}")
for name, question in TASKS.items():
    src = (EXAMPLES / f"{name}.bohm").read_text()
    stdin_file = EXAMPLES / f"{name}.in"
    stdin = stdin_file.read_text() if stdin_file.exists() else ""
    qa(question, prog(src, " ".join(stdin.split())))

out = HERE / "dataset" / "bohm.jsonl"
with out.open("w") as f:
    for row in rows:
        f.write(json.dumps(row, ensure_ascii=False) + "\n")
print(f"wrote {len(rows)} examples to {out}")
