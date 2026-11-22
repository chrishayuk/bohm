# Choices

The language has no "if". A program chooses by computing which group to
continue at.

## Zero or one

Two small expressions turn a number into a 0 or a 1:

| Expression | Value |
|------------|-------|
| `1 ∩ x` | 1 if x is not 0, otherwise 0 |
| `1 ∸ x` | 1 if x is 0, otherwise 0 |

`1 ∩ x` is the smaller of 1 and x. `1 ∸ x` is 1 minus x, which stops at 0.

## Choosing between two groups

Multiply each group by a 0 or a 1 and add. Whichever group is multiplied by
1 is the answer:

```bohm
π' → A
? → n
(1 ∩ n)·P + (1 ∸ n)·Z → π

π' → P              # n is not zero
1 → ?
Ω → π

π' → Z              # n is zero
0 → ?
Ω → π
```

```input
7
```

```output
1
```

One of `1 ∩ n` and `1 ∸ n` is always 1 and the other 0, so `π` receives
exactly `P` or exactly `Z`.

## Choosing whether to stop

If every multiplier is 0, `π` receives 0, and 0 is "no group": the program
stops. So a choice with one branch means "continue there, or else stop":

```text
(1 ∩ n)·C → π
```

## Comparisons

Every comparison is built from the two expressions above and the
subtraction that stops at zero.

| Condition | Is 1 when true |
|-----------|----------------|
| x ≠ 0 | `1 ∩ x` |
| x = 0 | `1 ∸ x` |
| x > y | `1 ∩ (x ∸ y)` |
| x ≤ y | `1 ∸ (x ∸ y)` |
| x < y | `1 ∩ (y ∸ x)` |
| x ≥ y | `1 ∸ (y ∸ x)` |
| x = y | `1 ∸ (x ÷ y)` |
| x ≠ y | `1 ∩ (x ÷ y)` |

`x ∸ y` is more than 0 only when x is greater than y. `x ÷ y`, the distance
between them, is 0 only when they are equal.

```bohm
π' → A
? → a
? → b
(1 ∩ (b ∸ a))·L + (1 ∸ (a ÷ b))·E + (1 ∩ (a ∸ b))·G → π

π' → L
1 → ?               # a is smaller
Ω → π

π' → E
2 → ?               # they are equal
Ω → π

π' → G
3 → ?               # a is greater
Ω → π
```

```input
8 8
```

```output
2
```

## Exactly one

The multipliers must never be 1 together. If two are, `π` receives the sum
of two groups, which is some unrelated place in the program.

Böhm made this very mistake in his own compiler. One of its groups ends

```text
[1 ∩ (r ∸ 1)]·D + (1 ∩ r)·E + (1 ∸ r)·F → π
```

and for r = 2, 3 or 4 both of the first two multipliers are 1. Nobody
noticed for as long as the program existed only on paper.

## And, or, not

With 1 for true and 0 for false:

| Logic | Arithmetic |
|-------|------------|
| p and q | `p·q` |
| p or q | `1 ∩ (p + q)` |
| not p | `1 ∸ p` |

A leap year is divisible by 4 and not by 100, or else divisible by 400:

```bohm
π' → A
? → y
1 ∸ (y mod 4) → a           # divisible by 4
1 ∩ (y mod 100) → b         # not divisible by 100
1 ∸ (y mod 400) → c         # divisible by 400
1 ∩ (a·b + c) → ?
Ω → π
```

```input
1900
```

```output
0
```

## Choosing a value

The same arithmetic picks a number instead of a group. This prints the
greater of two numbers without the `∪` operation, and without a jump:

```bohm
π' → A
? → a
? → b
(1 ∩ (a ∸ b))·a + (1 ∸ (a ∸ b))·b → ?
Ω → π
```

```input
12 31
```

```output
31
```
