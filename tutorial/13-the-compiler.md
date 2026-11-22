# The Compiler

The last chapter showed formulas turned into instruction numbers. The
program that does the turning is the heart of Böhm's dissertation, and it
is written in the language of this tutorial.

## What it reads

Böhm imagined a teletypewriter with a key for each symbol. Pressing a key
would punch a number on a tape. The number is 5·k + r, where r says what
kind of symbol it is and k which one:

| Symbol | r | k |
|--------|---|---|
| `)` | 0 | 6 |
| `(` | 1 | 6 |
| `→` | 2 | 5 |
| a variable | 3 | its address, plus 1000 for `↓` |
| an operation | 4 | its number |

`--emit chars` shows a program as that tape. The four lines of
`examples/echo.bohm` are twelve key presses, and a closing bracket marks
the end:

```console
$ bohm compile examples/echo.bohm --emit chars
4998
27
143
8
27
13
13
27
8
4993
27
3
30
```

The first number is 4998: variable (3) number 999, which is `π′`. Then 27
is the arrow and 143 is variable 28, the letter A.

## What it writes

Instruction numbers, on another tape, ready for the loader:

```console
$ bohm compile examples/echo.bohm --via bohm --emit code
00000509990028
00000500010002
00000500020001
00000509980000
```

`--via bohm` means the numbers were produced by Böhm's own compiler,
running as instructions on the simulated machine. Leave it out and they
come from a second translator, written in Rust from the prose of the
dissertation. The test suite compares the two, number for number, on the
dissertation's examples and on random formulas, and `bohm verify` does it
for any program:

```console
$ bohm verify examples/gcd-lcm.bohm --input 1071,462
formulas read as algebra:        output [21, 23562] in 23 steps
reference translation, machine:  output [21, 23562] in 52 steps
Böhm's compiler, machine:        output [21, 23562] in 52 steps
instruction numbers:             Böhm's compiler wrote 27, the reference 27: identical
all three agree
```

"Formulas read as algebra" is a third thing: the program run directly from
its formulas with no instruction numbers at all, as chapter 3 of the
dissertation says it can be understood.

## How it works

The compiler is 31 groups: fewer than a hundred formulas and a table of 25
jumps. It is in `programs/compiler-1951.bohm`. It reads one symbol at a time and never
looks back further than the symbol before.

For a polynomial it writes instructions as it goes, keeping the sum in `S`
and the current product in `X`. That is where chapter 4's reserved letters
come from.

For a bracketed formula it numbers each pair of brackets as it meets the
opening one, builds the instruction for each pair in a cell of its own, and
writes them out backwards when the formula ends, so that the innermost
brackets are computed first. The choice of what to do next is the table of
25 jumps from chapter 9, indexed by the kind of the last two symbols.

## It compiles itself

Being written in its own language, the compiler can be given its own text:

```console
$ bohm selfhost
chapter 7: 142 formulas, 1057 teletypewriter symbols, 401 instruction numbers
stage 0: chapter 7 read as algebra wrote 401 numbers: identical to the reference translation
stage 1: the stage 0 numbers, on the machine, wrote 401 numbers: identical to the reference translation
stage 2: the stage 1 numbers, on the machine, wrote 401 numbers: identical to the reference translation
stage 2 equals stage 0: the compiler reproduces itself
```

Stage 0 runs the compiler's formulas as algebra on its own text, which
gives the compiler as 401 instruction numbers. Stage 1 runs those numbers
on the machine, on the same text. Stage 2 runs the result of stage 1. All
of them agree with the independent translator, and stage 2 writes exactly
what stage 0 wrote.

The stages alternate between two arrangements of memory. The compiler keeps
its record of open brackets in the same cells where the code it writes
keeps intermediate results, so a compiler cannot be compiled with its own
setting.

## What was wrong with it

As printed in 1954, the compiler does not work. The tools make four
corrections, and with any one left out it cannot get through its own text.
Three are slips. The fourth, `names`, is needed because the tools compile
the compiler's formulas without brackets as chapter 6 polynomials, which
work in `X`. Put every formula in brackets instead and the compiler keeps
its `X` without being renamed; a test checks that. Each correction is
labelled with its kind. The transcription itself also supplies two arrows
that the printing leaves out:

```console
$ bohm errata
Corrections to chapter 7, switchable with --errata:

branch (slip)
    Group C chooses D and E at once
    The choice at the end of group C is [1∩(r∸1)]·D + (1∩r)·E + (1∸r)·F. For a variable, an operation or → (r = 2, 3, 4) both the first and the second coefficient are 1, so π receives D + E. The coefficient of E must be 1 only for an opening parenthesis: 1 ∸ (r÷1), as group A writes it for B.

store (slip)
    Group E′ leaves out the operation code of a store
    As printed, `a → b` is encoded as h·m3 + k, a number with operation code 00, which the machine does not define. Section 4.4 encodes the same formula as 5·m2 + C·m3 + F. One term is added.

link (slip)
    Group I overwrites the link to the enclosing parenthesis
    When a ( follows an operation, group I files the finished instruction number in the cell of the current parenthesis, which still holds F of the parenthesis around it. Knuth and Trabb Pardo (1976) report this bug without printing their fix; this one is ours, and it is not shown to be the smallest (they thought one more statement would do; this changes one and adds two). The instruction number is filed in the cell of the new parenthesis instead, where its last three digits still serve as the link, and group G moves it home when the outer parenthesis closes.

names (interpretation)
    The compiler's variable X is also the X of the code it generates
    Chapter 6 keeps the running product of a polynomial in the letter X (and the sum in S). Chapter 7 uses X for F(n) and is itself written with polynomials, so the compiler compiled by its own method destroys its X. Its X is renamed Z. This rests on reading the compiler's formulas without brackets as chapter 6 polynomials, as Knuth and Trabb Pardo describe Böhm's compiler doing; with every formula bracketed instead, X survives and the other three corrections suffice.

Corrections made in the transcriptions, always applied:

programs/compiler-1951.bohm, group J′ (French p. 43) (slip)
    Both formulas after π′ → J′ are printed without their arrow; the two arrows are supplied. Sestoft's translation (p. 41) has them.

programs/loader.bohm, group β (French p. 28) (slip)
    The constant is printed 94999010000, which does not recognise π′ → K; 9490010000 (Sestoft's reading) does, and is used.

programs/euclid-n.bohm, group F (French p. 26) (slip)
    The group described as F is headed π′ → E, a second E; it is written π′ → F, as the jumps to F and figure 3 require.

programs/tpk.bohm, group V (Knuth and Trabb Pardo, p. 39) (slip)
    Group V is headed π′ → U; it is written π′ → V.

Where the sources disagree (conflict):

table (6.2) and group G′ (French pp. 38, 43)
    The table begins a polynomial whose first term is a product with that product; group G′ first writes 0 → S. The reference translation follows G′.

group W (French p. 42)
    Sestoft's translation (p. 40) goes from group U to A′ and has no group W. The French printing's W is transcribed.

group F′ (French p. 43)
    The French prints h·m3 + g6 → ?, Sestoft's translation (p. 41) h·m2 + g6 → ?. The French is followed.

Kinds: slip, wrong under any reading of the source; conflict, the sources
disagree and one is followed; interpretation, our reading of what was meant;
ours, a consequence of choices made in this reconstruction.
```

The four are applied by default. `--errata none` runs the text as printed,
apart from those two arrows:

```console
$ bohm compile examples/echo.bohm --via bohm --errata none --emit code
00000009990028
00000000010002
00000000020001
00000009980000
```

Compare them with the four numbers above. Each has 00 where the operation
code 05 should be, so none of them is an instruction. That is the erratum
called `store`.

None of this takes anything from the achievement. The program was written
for a machine that did not exist, by a man who had no way to try it, and
three slips, two missing arrows and a clash of names in a program of this
size is fewer than most people make with a computer in front of them.

## A program that does not survive

Donald Knuth and Luis Trabb Pardo wrote a small test program, TPK, in
every early language they surveyed, Böhm's among them. Theirs is in
`programs/tpk.bohm`. Read as algebra it is correct. But it names a group
`S` and keeps a return address in `X`:

```console
$ bohm verify programs/tpk.bohm --input 0,1,2,3,4,5,6,7,8,9,10
formulas read as algebra:        output [10, 999, 9, 999, 8, 999, 7, 999, 6, 999, 5, 999, 4, 322, 3, 136, 2, 41, 1, 6, 0, 0] in 1775 steps
reference translation, machine:  output [] in 142 steps
Böhm's compiler, machine:        output [] in 142 steps
instruction numbers:             Böhm's compiler wrote 108, the reference 108: identical
bohm: the three readings do not agree
```

`programs/tpk-renamed.bohm` changes the two names and nothing else:

```console
$ bohm verify programs/tpk-renamed.bohm --input 0,1,2,3,4,5,6,7,8,9,10
formulas read as algebra:        output [10, 999, 9, 999, 8, 999, 7, 999, 6, 999, 5, 999, 4, 322, 3, 136, 2, 41, 1, 6, 0, 0] in 1775 steps
reference translation, machine:  output [10, 999, 9, 999, 8, 999, 7, 999, 6, 999, 5, 999, 4, 322, 3, 136, 2, 41, 1, 6, 0, 0] in 7731 steps
Böhm's compiler, machine:        output [10, 999, 9, 999, 8, 999, 7, 999, 6, 999, 5, 999, 4, 322, 3, 136, 2, 41, 1, 6, 0, 0] in 7731 steps
instruction numbers:             Böhm's compiler wrote 108, the reference 108: identical
all three agree
```
