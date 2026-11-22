# Reference

## Formulas

```text
expression → variable
expression → ↓variable
```

One formula per line. `#` begins a comment.

## Names

| Name | Meaning |
|------|---------|
| `a`–`z`, `A`–`Z` | variables and groups; avoid `S` and `X` |
| `m3`, `A'` | a letter with an index or primes: more names when needed |
| `?` | on the left, the next number of input; on the right, output |
| `π` | the group to continue at |
| `π′` | the group that begins here |
| `Ω` | no group; a cell that holds 0 |
| `↓x` | the cell whose address is in x |
| `12` | a number stands for a cell holding that number |

## Operations

| Symbol | Type | Result | Number |
|--------|------|--------|--------|
| `+` | `+` | sum, keeping the last 14 digits | 01 |
| `·` | `*` | product, keeping the last 14 digits | 02 |
| `∸` | `-` | difference, or 0 | 03 |
| `:` | `/` | quotient | 04 |
| `→` | `->` | copy | 05 |
| `÷` | `~` | absolute difference | 06 |
| `mod` | `mod` | remainder | 07 |
| `∪` | `max` | the greater | 08 |
| `∩` | `min` | the smaller | 09 |

`·` and `:` are done before `+` and `∸`; equal ranks from left to right.
`÷`, `mod`, `∪` and `∩` must stand alone or in brackets. `(`, `[` and `{`
are interchangeable.

Other spellings: `pi` for `π`, `pi'` for `π′`, `Omega` for `Ω`, `@x` for
`↓x`.

## Control

| Formula | Effect |
|---------|--------|
| `π' → K` | names the group that begins here K |
| `K → π` | continue at K |
| `Ω → π` | stop |
| `(1 ∩ c)·K → π` | continue at K if c is not 0, otherwise stop |
| `(1 ∩ c)·K + (1 ∸ c)·L → π` | continue at K if c is not 0, otherwise at L |
| `T + i → π` | continue i cells into group T |
| `z → π` | continue at the group whose name was stored in z |

The program starts at the first group in the file.

## Conditions

| Condition | Is 1 when true, 0 when false |
|-----------|-------------------------------|
| x ≠ 0 | `1 ∩ x` |
| x = 0 | `1 ∸ x` |
| x > y | `1 ∩ (x ∸ y)` |
| x ≤ y | `1 ∸ (x ∸ y)` |
| x = y | `1 ∸ (x ÷ y)` |
| x ≠ y | `1 ∩ (x ÷ y)` |
| p and q | `p·q` |
| p or q | `1 ∩ (p + q)` |
| not p | `1 ∸ p` |

## Directives

| Line | Effect | Default |
|------|--------|---------|
| `origin N` | load the program from cell N | 500 |
| `temps N` | intermediate results go to N+1, N+2, … | 80 |
| `cells N` | numbers and extra names go to N, N+1, … | 54 |
| `const NAME = N` | the cell NAME holds N at the start | |

## Limits

| | |
|---|---|
| Largest number | 99 999 999 999 999 |
| Memory | 1000 cells; 100–499 free for lists by default |
| Brackets in one formula | 19 pairs by default; more with `temps` |
| Numbers and extra names in a program | 27 by default; more with `cells` |
| Program size | 440 instructions by default; more with `origin` |

## When a program is stopped

| Message | Cause |
|---------|-------|
| `division by zero` | `:` or `mod` with a second operand of 0 |
| `input tape exhausted` | a `?` was read with no numbers left |
| `π = N is outside the memory` | a choice gave `π` a sum of groups, or a wrong value |
| `undefined instruction` | `π` pointed at a cell that does not hold an instruction |
| `step limit reached` | 50 million steps: probably a loop with no way out |

## Commands

| Command | Does |
|---------|------|
| `bohm run FILE` | compile with Böhm's compiler, load, run |
| `bohm run FILE --via interp` | run the formulas as algebra |
| `bohm run FILE --via ref` | compile with the reference translator, load, run |
| `bohm run FILE --trace` | also print each instruction executed |
| `bohm verify FILE` | run all three ways and compare |
| `bohm compile FILE` | list the instructions |
| `bohm compile FILE --emit normal` | show the formulas as the compiler accepts them |
| `bohm compile FILE --emit chars` | show the teletypewriter tape |
| `bohm compile FILE --emit code` | show the instruction numbers |
| `bohm selfhost` | have Böhm's compiler compile itself |
| `bohm errata` | list the corrections to the 1954 printing |

Input is given with `--input 3,4,5` or on standard input.
