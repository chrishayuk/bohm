# Idioms: doing without

The language has eight operations, whole numbers that never go below zero,
and no statement but assignment. This chapter collects the usual ways of
writing what it has no symbol for.

## Even or odd

`n mod 2` is 1 for an odd number and 0 for an even one, so it is already a
multiplier for a choice:

```bohm
π' → A
? → n
(n mod 2)·P + (1 ∸ (n mod 2))·Q → π

π' → P
1 → ?
Ω → π

π' → Q
2 → ?
Ω → π
```

```input
37
```

```output
1
```

The same works for "d divides n": `1 ∸ (n mod d)` is 1 when the remainder
is 0.

## Rounding

`:` rounds down. To round up, add one less than the divisor first. To round
to the nearest, add half the divisor:

```bohm
π' → A
? → a
? → b
a:b → ?
(a + b ∸ 1):b → ?
(a + b:2):b → ?
Ω → π
```

```input
17 5
```

```output
3
4
3
```

## Fractions

There are only whole numbers, so keep a quantity in smaller units. This
divides to two decimal places by working in hundredths, and prints the
whole part and the hundredths separately:

```bohm
π' → A
? → a
? → b
a·100:b → q
q:100 → ?
q mod 100 → ?
Ω → π
```

```input
22 7
```

```output
3
14
```

22/7 is 3.14 to two places.

## Below zero

`a ∸ b` is 0 whenever b is the larger. When a result can fall below zero,
keep its size and its sign apart. `a ÷ b` is the size of the difference
either way, and `1 ∩ (b ∸ a)` is 1 exactly when the difference is negative.
This prints both for a temperature in Fahrenheit converted to Celsius:

```bohm
π' → A
? → f
(f ÷ 32)·5:9 → ?
1 ∩ (32 ∸ f) → ?
Ω → π
```

```input
5
```

```output
15
1
```

5 °F is 15 below zero: size 15, sign 1.

## The digits of a number

`n mod 10` is the last digit and `n:10` is the rest. A loop that takes
digits off the right until nothing is left visits them all, here to add
them up:

```bohm
π' → A
? → n
0 → t

π' → B
(1 ∩ n)·C + (1 ∸ n)·D → π

π' → C
t + (n mod 10) → t
n:10 → n
B → π

π' → D
t → ?
Ω → π
```

```input
1951
```

```output
16
```

Building a number is the reverse: `r·10 + d → r` puts the digit d on the
right of r.

## Exchanging two variables

The plain way uses a third variable. Without one, add and subtract:

```bohm
π' → A
? → a
? → b
a + b → a
a ∸ b → b
a ∸ b → a
a → ?
b → ?
Ω → π
```

```input
17 42
```

```output
42
17
```

After the first line a holds the sum, so the second leaves the old a in b,
and the third leaves the old b in a.

## Keeping a condition

A choice needs each condition once for the group it picks and once,
reversed, for the other. When the condition is long, or when two conditions
must both hold, work it out into a variable first. Here c is 1 only when n
is between 10 and 20:

```bohm
π' → A
? → n
(1 ∸ (10 ∸ n))·(1 ∸ (n ∸ 20)) → c
c·P + (1 ∸ c)·Q → π

π' → P
1 → ?
Ω → π

π' → Q
0 → ?
Ω → π
```

```input
14
```

```output
1
```

Because c can only be 0 or 1, c itself is the multiplier and `1 ∸ c` is
its opposite.

## Three ways

Comparing two numbers has three outcomes. Give each its own multiplier,
and check that no two can be 1 together (chapter 6):

```bohm
π' → A
? → a
? → b
(1 ∩ (a ∸ b))·G + (1 ∩ (b ∸ a))·L + (1 ∸ (a ÷ b))·E → π

π' → G              # a is greater
1 → ?
Ω → π

π' → L              # a is less
2 → ?
Ω → π

π' → E              # they are equal
3 → ?
Ω → π
```

```input
4 9
```

```output
2
```

## A stack

Recursion, and anything else that must remember an unknown number of
things, needs a stack. Keep it in the free cells from 100, with a variable
holding the address of the first free cell. Two formulas put a number on
it, and two take the last one off:

```bohm
π' → A
100 → p
? → a
a → ↓p              # put a on the stack
p + 1 → p
? → a
a → ↓p              # and another
p + 1 → p
p ∸ 1 → p           # take the last one off
↓p → b
b → ?
p ∸ 1 → p
↓p → b
b → ?
Ω → π
```

```input
7 9
```

```output
9
7
```

The stack is empty when p is back at 100, which is the test
`1 ∩ (p ∸ 100)`.
