# Böhm's Language

In 1951 Corrado Böhm, a doctoral student at ETH Zürich, wrote a dissertation
that contained three things: a design for a computer, a language for writing
its programs as formulas, and a program, written in that language, that
turned formulas into machine instructions. It was the first description of a
compiler written in its own language. The computer was never built.

This tutorial teaches the language. It needs no knowledge of the
dissertation; chapters 12 and 13 go back to it and to the machine
underneath.

Three ideas explain most of what you will see in a program:

- **Every statement is an assignment.** `a + b → c` computes `a + b` and
  puts the result in `c`. There is no other kind of statement.
- **Going somewhere is an assignment too.** The variable `π` holds the place
  the program continues from. `B → π` continues at group B. Reading is
  assignment from `?` and printing is assignment to `?`.
- **A choice is arithmetic.** There is no "if". A program computes the place
  to continue from, with expressions that come out as one group or another.

Numbers are whole and never negative, and names are single letters.

## Running a program

Build the tools with Cargo, from the top folder of this repository:

```bash
cargo build --release
```

The program is then at `target/release/bohm`. The examples in this tutorial
write it as `bohm`. To run a program, give it a file and the numbers the
program will read:

```console
$ bohm run examples/gcd-lcm.bohm --input 1071,462
21
23562
```

Without `--input`, the numbers are read from standard input, so
`bohm run examples/gcd-lcm.bohm < examples/gcd-lcm.in` does the same.

## A first program

```bohm
π' → A
? → a
a·a → ?
Ω → π
```

```input
12
```

```output
144
```

The program reads a number, multiplies it by itself, and prints the result.
The first line gives the group a name and the last line stops. The next
chapter takes the four lines one at a time.

## Typing the symbols

The dissertation uses symbols that are not on a keyboard. Each has a
spelling that is:

| Symbol | Type | Meaning |
|--------|------|---------|
| `→` | `->` | becomes |
| `π` | `pi` | the group to continue at |
| `π′` | `pi'` | the group that begins here |
| `Ω` | `Omega` | no group |
| `↓` | `@` | the cell whose address is in |
| `∸` | `-` | minus, stopping at 0 |
| `·` | `*` | times |
| `:` | `/` | divided by |
| `÷` | `~` | difference |
| `∪` | `max` | the greater |
| `∩` | `min` | the smaller |

The first program can be typed like this, and means exactly the same:

```bohm
pi' -> A
? -> a
a*a -> ?
Omega -> pi
```

```input
12
```

```output
144
```

The rest of the tutorial uses the symbols.
