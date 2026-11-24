# bohm

Corrado Böhm's 1951 programming language, the three-address machine he
designed for it, his loader, and his compiler, which is written in the
language it compiles.

Böhm described all of it in his ETH Zürich dissertation, *Calculatrices
digitales. Du déchiffrage de formules logico-mathématiques par la machine
même dans la conception du programme* (1951, printed 1954). The machine was
never built. Here the compiler in chapter 7 runs on a simulation of the
machine in chapter 1, and compiles itself.

```
$ cargo run -- selfhost
chapter 7: 142 formulas, 1057 teletypewriter symbols, 401 instruction numbers
stage 0: chapter 7 read as algebra wrote 401 numbers: identical to the reference translation
stage 1: the stage 0 numbers, on the machine, wrote 401 numbers: identical to the reference translation
stage 2: the stage 1 numbers, on the machine, wrote 401 numbers: identical to the reference translation
stage 2 equals stage 0: the compiler reproduces itself
```

## The language

Every statement is an assignment, `expression → variable`. A program is a
set of groups; each begins `π′ → K`, which names it K.

| formula    | meaning                                   |
|------------|-------------------------------------------|
| `? → a`    | read a number into a                      |
| `a → ?`    | print a                                   |
| `B → π`    | continue at group B                       |
| `Ω → π`    | stop                                      |
| `? → ↓i`   | read into the cell whose address is in i  |

Numbers are non-negative integers of 14 decimal digits. The operations are
`+`, `·`, `∸` (difference, or 0), `:` (quotient), `÷` (absolute difference),
`mod`, `∪` (greater) and `∩` (smaller). There is no conditional: a choice is
arithmetic on group names, as in Euclid's algorithm from section 3.4:

```
π' → B
M mod m → r
{[(1 ∸ r)·C] + [(1 ∩ r)·D]} → π
```

ASCII spellings (`->`, `pi'`, `Omega`, `@i`, `-`, `*`, `/`, `~`, `max`,
`min`) are listed at the top of `src/syntax.rs`.

## Install

You need [Rust](https://rustup.rs):

```
cargo install --git https://github.com/chrishayuk/bohm
```

That puts `bohm` in `~/.cargo/bin`. Then:

```
echo "π' → A
6·7 → ?" > answer.bohm
bohm run answer.bohm
```

For Visual Studio Code, download `bohm-language-0.1.2.vsix` from the
[latest release](https://github.com/chrishayuk/bohm/releases/latest) and
install it:

```
code --install-extension bohm-language-0.1.2.vsix
```

Open any `.bohm` file and you get highlighting, hover help, problems as you
type, and run commands. The extension carries its own copy of the program,
so it works without the `cargo install` above. For Neovim, Helix and other
editors see [`editors/README.md`](editors/README.md).

Or try the language with nothing installed, in the
[playground](https://chrishayuk.com/demos/bohm).

## Using it

```
cargo run -- run programs/euclid.bohm --input 1071,462
cargo run -- verify programs/euclid-n.bohm --input 4,1071,462,84,147
cargo run -- compile programs/euclid.bohm
cargo run -- run programs/euclid.bohm --input 12,18 --trace
cargo run -- errata
cargo test
```

`run` compiles the program with Böhm's compiler running on the simulated
machine, loads the result with Böhm's loader, and runs it. `verify` runs a
program three ways and compares them.

## Examples

`examples/` holds 42 programs, from reading and printing a number to two
sorts, a binary search, a sieve, the Towers of Hanoi and Ackermann's
function with a hand-made stack, and a program that writes one of its own
instructions. Each has a `.out` file
with its expected output and, where it reads numbers, a `.in` file:

```
cargo run -- run examples/hanoi.bohm < examples/hanoi.in
```

`cargo test` runs every example on the machine and, except
`self-modifying.bohm` (marked `# machine only`: an instruction it builds
as a number has no meaning as algebra), as algebra too, and compares each
run with the `.out` file.

`programs/` holds the texts from the literature: Böhm's compiler and
loader, his two Euclid programs, and Knuth and Trabb Pardo's TPK.

## Tutorial and dataset

`tutorial/` is a 14-chapter tutorial, starting at
[`01-introduction.md`](tutorial/01-introduction.md). It teaches the
language first and comes to the machine and the compiler at the end. Every
program and every command in it is followed by what it prints, and

```
python3 tutorial/check_tutorial.py
```

runs them all and compares.

`tutorial/build_dataset.py` writes `tutorial/dataset/bohm.jsonl`, 80
questions and answers about the language in chat format. Every program in
an answer is run, and the output shown is what it printed.

## Playground

`web/` runs the same program in a browser. It is compiled unchanged for
WASI (`wasm32-wasip1`), and `web/wasi.js` is the small host it needs.

```
node web/build.mjs                  # builds web/bohm.wasm and web/examples.json
node web/test.mjs                   # runs every example through the wasm build
python3 -m http.server -d web 8000  # then open http://localhost:8000
```

The page can run a program on the machine or as algebra, show its
instructions, trace it, compare all three readings, run the compiler as
printed in 1954, and have the compiler compile itself.

Its editor colours the program, underlines problems before you run it, and
shows what a symbol means and what a formula compiles to when the pointer
rests on it. All of that comes from `bohm lsp` running in a worker
(`web/lsp-worker.js`), from the same `bohm.wasm`. Typing `->` writes `→`.

## Editor support

`editors/vscode/` is a VS Code extension: highlighting, snippets, the
symbols a keyboard lacks, problems as you type, the instruction numbers
behind a formula when you rest the pointer on it, go to a group's
definition, rename, and commands to run the file. It carries the program
compiled to WebAssembly, so it needs nothing installed.

The extension is thin. The work is done by three commands of the same
program, which any editor can use ([`editors/README.md`](editors/README.md)):

```
bohm check FILE     # problems, one per line: FILE:LINE:COLUMN: KIND: MESSAGE
bohm format FILE    # the program with -> pi Omega - * written as → π Ω ∸ ·
bohm lsp            # a language server on standard input and output
```

`check` reports more than syntax. It warns when a program uses `S` or `X`
beside a formula without brackets, which is what stops Knuth and Trabb
Pardo's TPK program on the machine, and what the fourth correction to
Böhm's own compiler is about:

```
$ bohm check programs/tpk.bohm | wc -l
5
$ bohm check programs/compiler-1951.bohm | wc -l
11
```

`web/lsp-worker.js` runs the same server in a web page.

## Three readings of a program

The thesis gives its notation two meanings: instructions for a machine
(chapter 2) and algebra with no machine in it (chapter 3). The tools follow
that split.

| reading | what it is | file |
|---|---|---|
| algebra | each formula evaluated directly, one step per formula | `src/interp.rs` |
| reference translation | chapters 5 and 6 written in Rust from the prose and tables | `src/compile.rs` |
| Böhm's compiler | chapter 7, transcribed, running as instruction numbers on the machine | `programs/compiler-1951.bohm` |

The reference translator is not a port of chapter 7. Agreement between the
two, number for number, is the test: on the dissertation's programs, on
random formulas, and on the compiler's own text.

The dissertation does not say in what order a formula reads its operands,
which matters only when it reads `?` more than once. All three readings use
the order of the machine's instructions: a polynomial left to right, a
bracketed formula in the order of (5.32), the last bracket opened first.
That rule is ours.

## What running it found

Each correction the repository makes is labelled with its kind:

* **slip**: wrong under any reading of the source;
* **conflict**: the sources say two things, and one is followed;
* **interpretation**: our reading of what was meant;
* **ours**: a consequence of choices made in this reconstruction.

`bohm errata` lists them with the same labels.

### Chapter 7

Chapter 7 is transcribed from the French printing and kept as printed,
with one exception: group J′ (p. 43) prints its two formulas without
arrows, and the transcription supplies them (slip; Sestoft's translation
has them). It does not work as printed. `bohm errata` lists four
corrections, applied by default and switchable with `--errata`. With any
one of them left out, the compiler cannot compile itself, given that its
formulas without brackets are compiled as chapter 6 polynomials.

| erratum | kind | problem | known before? |
|---|---|---|---|
| `branch` | slip | Group C's choice gives π the sum D + E for every symbol that is not a parenthesis. | not that we found |
| `store` | slip | Group E′ encodes `a → b` without the operation code 05. | not that we found |
| `link` | slip | Group I overwrites the link to the enclosing parenthesis when `(` follows an operation. | Knuth and Trabb Pardo (1976) report it, without printing a fix. The fix here is ours and is not shown to be the smallest: they thought one more statement would do, and this one changes one statement and adds two |
| `names` | interpretation | The compiler keeps F(n) in X, and chapter 6 computes every polynomial in X and S. The compiler compiled by its own method destroys its own X. | Section 6.2 names the two letters, and Sestoft's appendix A.1 notes that S, V, X and Y are spoken for. Neither says the compiler's X and the polynomial X collide |

`names` is an interpretation because it depends on how the compiler's own
formulas are compiled. Read as chapter 6 polynomials, as Knuth and Trabb
Pardo describe Böhm's compiler reading a formula that does not begin with a
bracket, they clash with X. Put every one of them in brackets instead and
the compiler compiles itself with the other three corrections alone; a test
(`names_is_needed_only_for_polynomials`) checks this. Renaming X is the
smaller change to the text, so it is the one made.

"Not that we found" means no earlier report in the sources listed below,
or in the literature search made for the independent review
([`docs/independent-review.md`](docs/independent-review.md)). It is not a
proof that nobody saw these before. Sestoft says he implemented the
compiler; his translation does not say which corrections he needed.

### Other transcriptions

* `programs/loader.bohm`: group β prints the constant 94999010000, which
  recognises no group header; 9490010000, as in Sestoft's translation, is
  used (slip). The groups are headed `π′` where the printing has `π″`, which
  footnote 20 says is only for clarity; three formulas are put in brackets
  so that the loader computes no polynomial; its letters are renamed and it
  is moved from 970 to 940 (all ours). The file lists each change.
* `programs/euclid-n.bohm`: the group described as F is headed `π′ → E` on
  p. 26; it is written `π′ → F` (slip). Group E falls through into F, as
  printed.
* `programs/tpk.bohm`: Knuth and Trabb Pardo head group V with `π′ → U` on
  their p. 39; it is written `π′ → V` (slip).

### Where the sources disagree

* Table (6.2) begins a polynomial whose first term is a product with the
  product itself; group G′ first writes `0 → S`. The reference translation
  follows G′ (conflict).
* Sestoft's translation omits group W of chapter 7 and gives group F′ as
  `h·m2 + g6 → ?`. The French printing has W and `h·m3 + g6 → ?`, and is
  followed (conflict).

### Left uncorrected

* The syntax checks at the end of groups A′ and C′ multiply a condition by Ω,
  which holds 0, so they never stop the machine. `a mod b → c` is accepted
  and compiled although section 6.12 excludes it. Knuth and Trabb Pardo
  remark that the syntax check "was not complete".
* Knuth and Trabb Pardo's TPK program in Böhm's language (`programs/tpk.bohm`)
  is correct as algebra, but it names a group S and a variable X. Compiled by
  Böhm's method it stops at once with no output.
  `programs/tpk-renamed.bohm` changes the two names and runs (ours: putting
  its formulas in brackets would also do). The reservation itself is known:
  Sestoft's translation notes that "the group labels S and V are not used in
  the chapter 7 compiler since they are used in the translation of
  polynomials (chapter 6)" (appendix A.1, p. 47). A search in November 2026
  found no one connecting it with TPK, and Knuth's errata for the reprint
  leave the Böhm program unchanged.

A compiler cannot be compiled with its own base constant m: it keeps its
parenthesis stack in the cells above m, and the code it writes keeps
intermediate results in the same cells. So self-compilation here alternates
two bases, and the fixed point is a cycle of length two that differs only in
those addresses. That is a property of this arrangement of memory (ours),
not a claim about the only one possible.

## What is not in the thesis

Each is labelled with its kind.

* **Constants** (ours). Section 2.6 writes a number as the address of a
  cell that holds it, but the thesis gives no way of putting the numbers in
  those cells before a program runs. Following Peter Sestoft's appendix, a
  small group ρ in front of the loader reads (address, number) pairs from
  the tape.
* **The decision table as a group** (ours). Section 5.7 says cells
  t..t+24 hold the 25 instructions of table (5.61). They are written here
  as a group named t.
* **Names beyond 52 letters** (ours). Chapter 7 needs primed groups and
  named constants, which the keyboard of section 4.3 cannot type. They get
  addresses from a `cells` directive, as section 2.5 allows; the compiler
  itself never looks at the range of a variable's number.
* **Bracketing** (interpretation). The thesis writes formulas its compiler
  does not accept (footnote 18). The tools bracket every operation of such a formula before
  compiling; `bohm compile --emit normal` shows the result.
* **The loader's position, variable names and brackets** (ours), noted in
  `programs/loader.bohm`.
* **The values of g2..g6** (interpretation). Section 6.3 gives their
  pattern and the value of g1 only; the others are written out from the
  pattern.
* **The order in which a formula reads `?`** (ours), described above.
* **Undefined behaviour** (ours). Division by zero, an exhausted tape, an
  unknown operation code and π ≥ 1000 stop the simulator with a message.

## Earlier work

Peter Sestoft's 2016 English translation says he implemented the machine,
loader and compiler in Java. The code does not appear on his page, and the
translation does not say which corrections it needed. This project was
written without it.

The repository was reviewed by a second model, OpenAI's Codex, in November
2026. What it found and what was done about each finding is in
[`docs/independent-review.md`](docs/independent-review.md).

## Sources

The PDFs are not in this repository. `docs/scans/` has reduced images of
the pages that the corrections rest on.

* Corrado Böhm, *Calculatrices digitales…*, Bologna 1954 (French original,
  ETH e-collection, eth-32719-02).
* The same, translated by Peter Sestoft, 2016,
  <https://raspi.itu.dk/~sestoft/boehmthesis/>.
* Donald E. Knuth and Luis Trabb Pardo, *The Early Development of
  Programming Languages*, Stanford STAN-CS-76-562, 1976, pp. 35–42.

## Licence

Apache License 2.0 - see [LICENSE](LICENSE). That covers the code, the
examples and the tutorial. The page images in `docs/scans` are reproduced
for reference and remain the copyright of their authors and publishers.
