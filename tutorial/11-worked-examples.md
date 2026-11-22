# Worked examples

Five complete programs, each built from the pieces of the earlier
chapters. All of them are in `examples/` as well.

## A power, by squaring

The subroutine of chapter 9 multiplied e times to raise b to the power e.
This takes about as many steps as e has binary digits. Whenever e is odd
the result is multiplied by b; then b is squared and e is halved.

```bohm
π' → A
? → b
? → e
1 → r

π' → B
(1 ∩ e)·C + (1 ∸ e)·E → π

π' → C              # If e is odd, multiply the result by b.
(e mod 2)·D + (1 ∸ (e mod 2))·F → π

π' → D
r·b → r

π' → F              # Square b and halve e.
b·b → b
e:2 → e
B → π

π' → E
r → ?
Ω → π
```

```input
3 13
```

```output
1594323
```

Group D has no jump at its end, so it falls through into F (chapter 5).
That is what makes the multiplication optional.

## Prime factors

Divide by 2 for as long as it divides, then by 3, and so on. There is no
need to go past the square root of what is left: if nothing up to there
divides it, it is a prime itself.

```bohm
π' → A
? → n
2 → d

π' → B              # While d·d <= n, d may divide n.
(1 ∸ (d·d ∸ n))·C + (1 ∩ (d·d ∸ n))·F → π

π' → C
n mod d → r
(1 ∸ r)·D + (1 ∩ r)·E → π

π' → D              # d divides n: print it and divide it out.
d → ?
n:d → n
B → π

π' → E
d + 1 → d
B → π

π' → F              # What is left is 1 or a prime.
(1 ∩ (n ∸ 1))·G → π

π' → G
n → ?
```

```input
360360
```

```output
2
2
2
3
3
5
7
11
13
```

The last choice has only one group in it. When n is 1 the multiplier is 0,
`π` receives 0, and the program stops.

## The day of the week

Zeller's rule gives the day of the week from the date. It treats January
and February as months 13 and 14 of the year before, which in most
languages is an "if". Here c is 1 for those two months and 0 otherwise,
and the adjustment is two formulas with no choice in them.

```bohm
π' → A
? → d
? → m
? → y
1 ∸ (m ∸ 2) → c
m + 12·c → m
y ∸ c → y
y mod 100 → k
y:100 → j
d + 13·(m + 1):5 + k + k:4 + j:4 + 5·j → h
((h + 5) mod 7) + 1 → ?
Ω → π
```

```input
20 7 1969
```

```output
7
```

The answer counts from 1 for Monday, so 20 July 1969 was a Sunday. In
`13·(m + 1):5` the multiplication is done first and then the division,
which rounds down.

## Binary search

To find a number in a sorted list, look at the middle and discard the half
it cannot be in. The comparison has three outcomes, so the choice in group
H has three multipliers.

The input is the length of the list, the list, and the number to find. The
program prints its position, counting from 1, or 0.

```bohm
π' → A
? → n
100 → i
100 + n → e

π' → B              # Read the list into 100 .. e-1.
(1 ∩ (e ∸ i))·C + (1 ∸ (e ∸ i))·D → π

π' → C
? → ↓i
i + 1 → i
B → π

π' → D              # The number, if present, is in l .. h-1.
? → v
100 → l
e → h
0 → r

π' → G              # While l < h.
(1 ∩ (h ∸ l))·H + (1 ∸ (h ∸ l))·N → π

π' → H              # Too small: I. Too large: J. Found: K.
(l + h):2 → k
↓k → u
(1 ∩ (v ∸ u))·I + (1 ∩ (u ∸ v))·J + (1 ∸ (u ÷ v))·K → π

π' → I
k + 1 → l
G → π

π' → J
k → h
G → π

π' → K
k ∸ 99 → r

π' → N
r → ?
Ω → π
```

```input
8  4 7 15 31 35 65 89 92
65
```

```output
6
```

65 is the sixth of the eight numbers, found after looking at three.

## Ackermann's function

Ackermann's function is defined by calling itself, twice over in its last
line:

```text
A(0, n) = n + 1
A(m, 0) = A(m − 1, 1)
A(m, n) = A(m − 1, A(m, n − 1))
```

The language has no recursion, but all a call has to remember here is the
m it will need afterwards. Those go on a stack (chapter 10), and the
program runs until the stack is empty.

```bohm
π' → A
? → m
? → n
100 → p
m → ↓p
p + 1 → p

π' → B              # While the stack is not empty, take the top m.
(1 ∩ (p ∸ 100))·C + (1 ∸ (p ∸ 100))·E → π

π' → C
p ∸ 1 → p
↓p → m
(1 ∸ m)·D + (1 ∩ m)·F → π

π' → D              # m = 0
n + 1 → n
B → π

π' → F
(1 ∸ n)·G + (1 ∩ n)·H → π

π' → G              # n = 0: A(m − 1, 1) is next.
m ∸ 1 → ↓p
p + 1 → p
1 → n
B → π

π' → H              # Otherwise A(m, n − 1) is next, and m − 1 waits.
m ∸ 1 → ↓p
p + 1 → p
m → ↓p
p + 1 → p
n ∸ 1 → n
B → π

π' → E
n → ?
Ω → π
```

```input
2 3
```

```output
9
```

## More programs to read

`examples/` has 42 programs. Some that go further than this chapter:

| Program | Shows |
|---------|-------|
| `hanoi.bohm` | recursion with five saved values for each call |
| `insertion-sort.bohm`, `bubble-sort.bohm` | two ways to sort a list |
| `pascal-triangle.bohm` | a row rebuilt in place |
| `josephus.bohm` | a circle of flags |
| `sieve.bohm` | the sieve of Eratosthenes |
| `jump-table.bohm` | a calculator that jumps through a table |
| `change.bohm` | a list ended by a 0 |
| `random.bohm` | a pseudo-random sequence |
| `self-modifying.bohm` | a program that writes one of its own instructions |
