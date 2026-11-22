# Subroutines

Böhm does not mention subroutines, but his language has what they need,
because a group name is a number that can be stored (chapter 5).

## Coming back

A subroutine is a group that ends by continuing at whatever group is named
in an agreed variable. The caller puts the place to come back to in that
variable and jumps.

Here group P computes b to the power e and leaves it in r. It returns to
the group named in z.

```bohm
π' → A
2 → b
10 → e
B → z               # come back at B
P → π

π' → B
r → ?
3 → b
4 → e
C → z               # come back at C
P → π

π' → C
r → ?
Ω → π

π' → P              # r = b to the power e. Returns to the group in z.
1 → r

π' → Q
(1 ∩ e)·R + (1 ∸ e)·z → π

π' → R
r·b → r
e ∸ 1 → e
Q → π
```

```output
1024
81
```

The return is in group Q: when e reaches 0 the choice picks `z`.

The subroutine's quantities are ordinary variables, shared with everything
else. Decide which letters belong to it, and keep them for it.

## A table of jumps

When there are many branches, a choice with a multiplier for each grows
long. A table is shorter. Each line of group T below is one jump, and each
jump takes exactly one cell, so the jump for operation o is at T + o − 1.

```bohm
π' → A              # Read an operation: 1 add, 2 multiply, 3 subtract,
? → o               # 4 divide. 0 stops.
(1 ∩ o)·B → π

π' → B
? → a
? → b
T + o ∸ 1 → π

π' → T
P → π
M → π
D → π
Q → π

π' → P
a + b → ?
A → π

π' → M
a·b → ?
A → π

π' → D
a ∸ b → ?
A → π

π' → Q
a:b → ?
A → π
```

```input
1 19 23
2 6 7
3 100 58
4 1951 7
0
```

```output
42
42
42
278
```

Assigning `T + o ∸ 1` to `π` continues in the middle of a group. Böhm's
compiler does exactly this to decide what to do with each symbol it reads:
it has a table of 25 jumps and continues at `5·r + s + t`.

Only a plain jump, `P → π`, is sure to take one cell. A table of anything
else would not line up.

## Subroutines that call themselves

A subroutine that calls itself must save its return group first, or the
second call overwrites it. There is no stack in the language, so the
program keeps one in a list (chapter 8).

This is the Towers of Hanoi. To move n discs from peg f to peg t: move
n − 1 discs out of the way, move the largest, move the n − 1 back on top.
Each move is printed as two digits, so `13` is "from peg 1 to peg 3".

```bohm
π' → A
? → n
1 → f
3 → t
2 → v
100 → p             # p is the address of the first free cell of the stack
E → z

π' → H              # Move n discs from f to t using v, then continue at z.
(1 ∸ n)·z + (1 ∩ n)·I → π

π' → I              # Save n, f, t, v and z. Move n-1 discs from f to v,
n → ↓p              # and come back at J.
p + 1 → p
f → ↓p
p + 1 → p
t → ↓p
p + 1 → p
v → ↓p
p + 1 → p
z → ↓p
p + 1 → p
n ∸ 1 → n
t → u
v → t
u → v
J → z
H → π

π' → J              # Recall n, f, t and v. Move the largest disc.
p ∸ 5 → q
↓q → n
q + 1 → q
↓q → f
q + 1 → q
↓q → t
q + 1 → q
↓q → v
f·10 + t → ?
n ∸ 1 → n           # Move n-1 discs from v to t, and come back at K.
f → u
v → f
u → v
K → z
H → π

π' → K              # Recall z, discard the saved numbers, and return.
p ∸ 1 → p
↓p → z
p ∸ 4 → p
z → π

π' → E
Ω → π
```

```input
3
```

```output
13
12
32
13
21
23
13
```
