# Loops

A loop is a group that jumps back to itself, or to a group before it, with a
choice that sooner or later goes somewhere else.

## Counting down

Count with a variable that goes down to 0, and test it with `1 ∩ n`:

```bohm
π' → A
? → n
0 → s

π' → B              # Add n to the sum, decrease it, repeat until it is 0.
s + n → s
n ∸ 1 → n
(1 ∩ n)·B + (1 ∸ n)·C → π

π' → C
s → ?
Ω → π
```

```input
100
```

```output
5050
```

That loop does its work before testing, so it runs at least once. To allow
for no turns at all, test first:

```bohm
π' → A
? → n
1 → f

π' → B              # While n > 0 continue at C, otherwise at D.
(1 ∩ n)·C + (1 ∸ n)·D → π

π' → C
f·n → f
n ∸ 1 → n
B → π

π' → D
f → ?
Ω → π
```

```input
10
```

```output
3628800
```

## Counting up

To count up to a limit, compare with it. `1 ∸ (i ∸ n)` is 1 while i is not
greater than n:

```bohm
π' → A
? → n
1 → i

π' → B              # While i <= n. Otherwise stop.
(1 ∸ (i ∸ n))·C → π

π' → C
i·i → ?
i + 1 → i
B → π
```

```input
6
```

```output
1
4
9
16
25
36
```

## Until something happens

The Collatz sequence halves an even number and triples-and-adds-one an odd
number, until it reaches 1:

```bohm
π' → A
? → n

π' → B              # Print n. Stop when it is 1.
n → ?
(1 ∩ (n ∸ 1))·C → π

π' → C              # r is 0 for an even number and 1 for an odd one.
n mod 2 → r
(1 ∸ r)·D + r·E → π

π' → D
n:2 → n
B → π

π' → E
3·n + 1 → n
B → π
```

```input
6
```

```output
6
3
10
5
16
8
4
2
1
```

`r` is already 0 or 1, so it is used as a multiplier directly.

## A loop inside a loop

The inner loop is just more groups. This prints the primes up to n: for each
p, it tries divisors d while d·d ≤ p.

```bohm
π' → A
? → n
2 → p

π' → B              # While p <= n.
(1 ∸ (p ∸ n))·C → π

π' → C
2 → d

π' → D              # While d·d <= p continue at E; if no d divided, at F.
(1 ∸ (d·d ∸ p))·E + (1 ∩ (d·d ∸ p))·F → π

π' → E              # If d divides p it is not prime: continue at H.
p mod d → r
(1 ∩ r)·G + (1 ∸ r)·H → π

π' → G
d + 1 → d
D → π

π' → F
p → ?

π' → H
p + 1 → p
B → π
```

```input
30
```

```output
2
3
5
7
11
13
17
19
23
29
```

Group F falls through into H.

## Two numbers at a time

Fibonacci numbers need the last two values, and a third variable while they
change places:

```bohm
π' → A
? → n
0 → a
1 → b

π' → B
(1 ∩ n)·C → π

π' → C
a → ?
a + b → c
b → a
c → b
n ∸ 1 → n
B → π
```

```input
10
```

```output
0
1
1
2
3
5
8
13
21
34
```
