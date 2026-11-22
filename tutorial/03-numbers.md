# Numbers and Operations

## The numbers

Every number is a whole number from 0 to 99 999 999 999 999: fourteen
decimal digits. There are no negative numbers and no fractions. Böhm chose
this to keep the machine simple, and showed that nothing is lost: a
negative number or a fraction can always be kept as a pair of whole ones.

## The eight operations

| Operation | Symbol | Result |
|-----------|--------|--------|
| sum | `a + b` | a + b |
| product | `a·b` | a × b |
| difference | `a ∸ b` | a − b, or 0 if b is greater |
| quotient | `a:b` | a ÷ b, with the remainder thrown away |
| remainder | `a mod b` | what is left after dividing a by b |
| absolute difference | `a ÷ b` | the distance between a and b |
| greater | `a ∪ b` | the greater of a and b |
| smaller | `a ∩ b` | the smaller of a and b |

Note that `:` divides, and `÷` does not.

```bohm
π' → A
? → a
? → b
a + b → ?
a·b → ?
a ∸ b → ?
b ∸ a → ?
a:b → ?
a mod b → ?
a ÷ b → ?
a ∪ b → ?
a ∩ b → ?
Ω → π
```

```input
17 5
```

```output
22
85
12
0
3
2
12
17
5
```

## Subtraction stops at zero

Because there are no negative numbers, `5 ∸ 17` is 0. This is not a
nuisance to work around: it is the most useful operation in the language.
`a ∸ b` is 0 exactly when a is not greater than b, and chapter 6 builds
every comparison from that.

## Too large

A sum or product that needs more than fourteen digits keeps the last
fourteen:

```bohm
π' → A
99999999999999 + 2 → ?
10000000·10000000 → ?
Ω → π
```

```output
1
0
```

## Division by zero

The dissertation does not say what dividing by zero does. Here it stops the
program:

```bohm
π' → A
? → a
100:a → ?
100 mod a → ?
Ω → π
```

```input
0
```

```output
bohm: stopped: division by zero
```

## Order of operations

In a formula without brackets, `·` and `:` are done before `+` and `∸`, and
operations of the same rank from left to right:

```bohm
π' → A
2 + 3·4 → ?         # 14, not 20
100 ∸ 30 ∸ 20 → ?   # 50, not 90
60:4·3 → ?          # 45, not 5
Ω → π
```

```output
14
50
45
```

Brackets change the order. Round, square and curly brackets all mean the
same; the dissertation mixes them so that long formulas are easier to read.

```bohm
π' → A
(2 + 3)·4 → ?
100 ∸ (30 ∸ 20) → ?
60:[4·3] → ?
Ω → π
```

```output
20
90
5
```

The other four operations, `÷`, `mod`, `∪` and `∩`, have no rank. The
dissertation only ever writes them alone or inside brackets, and so must
you:

```bohm
π' → A
? → n
n mod 10 → ?            # alone: fine
(n mod 10) + 1 → ?      # combined: brackets needed
Ω → π
```

```input
1951
```

```output
1
2
```

Leave the brackets out and the program is refused:

```bohm
π' → A
? → n
n mod 10 + 1 → ?
Ω → π
```

```input
1951
```

```output
bohm: program.bohm: line 3: put brackets around the `mod` operation
```
