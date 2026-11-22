# The Machine

Everything so far could be read as algebra, with no machine in mind. Böhm
insists on that: his third chapter argues that the notation describes a
method of calculation, not the behaviour of a computer. But the same
formulas are also instructions for the computer of his first chapter, and
this chapter looks at them that way.

## The computer

- A memory of about 1000 cells, numbered from 000. Each holds a number of
  fourteen decimal digits.
- Cell 000 is the register `π`. It holds the address of the next
  instruction.
- Cell 001 is the tape reader when it is read and the printer when it is
  written. That is `?`.
- Cells 002 to 053 are the letters `a` to `z` and `A` to `Z`.
- Cell 998 is `Ω` and always holds 0. Cell 999 is `π′`.

The machine does one thing for ever: fetch the number in the cell `π`
points at, add 1 to `π`, and carry out that number as an instruction.

## Instructions are numbers

An instruction is a fourteen-digit number in four parts:

```text
0002  01  0003  0004
 │    │    │     └── where the result goes        cell 4, c
 │    │    └──────── the second operand           cell 3, b
 │    └───────────── the operation                01, +
 └────────────────── the first operand            cell 2, a
```

That number is `a + b → c`. The operations are numbered 01 `+`, 02 `·`,
03 `∸`, 04 `:`, 06 `÷`, 07 `mod`, 08 `∪`, 09 `∩`. Operation 05 copies the
second operand to the result and ignores the first: `a → b`.

The first digit of each address is 0 for the cell itself and 1 for `↓`.

A jump is nothing special. `B → π` is a copy into cell 000. Reading and
printing are copies from and to cell 001. The number 0 is "stop", which is
why giving `π` the value 0 stops the machine: the next instruction is
fetched from cell 000, and it is 0.

## Seeing the instructions

`bohm compile` shows what a program becomes:

```console
$ bohm compile examples/countdown.bohm
# named cells            054..054
# intermediate results   081..082
# instructions           500..508
     0000 05 0999 0028   π′ → A
500  0000 05 0001 0015   ? → n
     0000 05 0999 0029   π′ → B
501  0000 05 0015 0001   n → ?
502  0054 09 0015 0082   ‹1›∩n → [082]
503  0082 02 0030 0081   [082]·C → [081]
504  0000 05 0081 0000   [081] → π
     0000 05 0999 0030   π′ → C
505  0000 05 0015 0046   n → S
506  0046 03 0054 0046   S∸‹1› → S
507  0000 05 0046 0015   S → n
508  0000 05 0029 0000   B → π
```

Each line is an address, the instruction number in its four parts, and the
same instruction in the notation. The lines without an address are the
`π′` lines: they are not instructions, and the next section says what
happens to them.

The cells `[081]`, `[082]`, … are where a bracketed formula keeps its
intermediate results.

## Loading

How does a group name come to hold the right address? Not through the
compiler, which never knows where a program will be put. Böhm's answer is a
small program, the loader, which reads instruction numbers from the tape
and puts them in consecutive cells. When it meets a `π′ → K` it does not
store it. It puts the address of the next free cell into K.

So `B → π` works wherever the program is loaded, and no instruction ever
has to be corrected afterwards. Every jump goes through a variable.

## Watching it run

`--trace` prints each instruction as it is executed. The two numbers at
the top are what the program printed:

```console
$ bohm run examples/countdown.bohm --input 1 --trace
1
0
500  0000 05 0001 0015   ? → n
501  0000 05 0015 0001   n → ?
502  0054 09 0015 0082   ‹1›∩n → [082]
503  0082 02 0030 0081   [082]·C → [081]
504  0000 05 0081 0000   [081] → π
505  0000 05 0015 0046   n → S
506  0046 03 0054 0046   S∸‹1› → S
507  0000 05 0046 0015   S → n
508  0000 05 0029 0000   B → π
501  0000 05 0015 0001   n → ?
502  0054 09 0015 0082   ‹1›∩n → [082]
503  0082 02 0030 0081   [082]·C → [081]
504  0000 05 0081 0000   [081] → π
000  0000 00 0000 0000   stop
```

## The memory

| Cells | Use |
|-------|-----|
| 000 | `π` |
| 001 | `?` |
| 002–053 | the letters |
| 054–080 | the numbers written in the program, and names that are not single letters |
| 081–099 | intermediate results of bracketed formulas |
| 100–499 | free for lists |
| 500–939 | the program |
| 940–997 | the loader |
| 998, 999 | `Ω`, `π′` |

A program that needs a different arrangement says so on lines of its own:

```text
origin 200          the program is loaded from cell 200
temps 100           intermediate results go to 101, 102, ...
cells 900           numbers and extra names go to 900, 901, ...
const m = 850       cell m holds 850 when the program starts
```

`const` is also how a program gets a number into a named cell without an
instruction. A name may carry an index or a prime, such as `m3` or `A'`,
when 52 letters are not enough; Böhm's compiler has groups A′ to N′.

## A program that changes itself

Since instructions are numbers in cells, and `↓` can reach any cell, a
program can write its own instructions. This one reads an operation number
and two operands, builds the instruction for "a, that operation, b, into c",
and puts it at the start of group P:

```bohm
# machine only
π' → A
? → o
? → a
? → b
2·10000000000 + o·100000000 + 30004 → ↓P
P → π

π' → P
Ω → Ω               # replaced before it is reached
c → ?
Ω → π
```

```input
7 1951 100
```

```output
51
```

Operation 7 is `mod`, and 1951 mod 100 is 51. `P` holds the address where
group P begins, so `↓P` is the cell of its first instruction.

This program has no meaning as algebra. Formulas are not numbers there, and
nothing can overwrite one. It is the one place in this tutorial where the
two readings of the notation part company.
