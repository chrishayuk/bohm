# Indexed Quantities

Mathematics writes a list of numbers as a₁, a₂, a₃, … and speaks of "the
quantity with index i". Böhm's notation for that is `↓i`.

## Cells and addresses

The machine's memory is a row of about a thousand numbered cells, and every
variable is one of them: `a` is cell 2, `b` is cell 3, and so on. The number
of a cell is its address.

`↓i` means "the cell whose address is the number in i". If `i` holds 100,
then `↓i` is cell 100.

```bohm
π' → A
100 → i
7 → ↓i              # put 7 in cell 100
101 → i
9 → ↓i              # put 9 in cell 101
100 → i
↓i → ?              # print cell 100
Ω → π
```

```output
7
```

## Where to keep a list

Cells 100 to 499 are free for your numbers. Below 100 are the letters and
the cells the compiler uses; from 500 up is the program itself. Böhm's rule
is the same: an index is a three-digit number, 100 or more.

So the i-th number of a list is kept at 100 + i, and the variable that walks
along the list holds the address, not the position.

## Reading a list and printing it backwards

```bohm
π' → A
? → n
100 → i
100 + n → e         # e is the address after the last number

π' → B              # Read while i < e.
(1 ∩ (e ∸ i))·C + (1 ∸ (e ∸ i))·D → π

π' → C
? → ↓i
i + 1 → i
B → π

π' → D              # Print while i > 100.
(1 ∩ (i ∸ 100))·E → π

π' → E
i ∸ 1 → i
↓i → ?
D → π
```

```input
5  10 20 30 40 50
```

```output
50
40
30
20
10
```

## Only a name can follow ↓

`↓` is followed by one letter, never by an expression. To reach cell
100 + k, work out the address first:

```text
100 + k → j
↓j → x
```

## Two lists

Two lists need two stretches of cells that do not overlap. This program
reads n numbers into 100… and writes their running totals into 200…:

```bohm
π' → A
? → n
100 → i
200 → j
100 + n → e
0 → t

π' → B
(1 ∩ (e ∸ i))·C + (1 ∸ (e ∸ i))·D → π

π' → C
? → ↓i
t + ↓i → t
t → ↓j
i + 1 → i
j + 1 → j
B → π

π' → D              # Print the second list.
200 → j
200 + n → e

π' → E
(1 ∩ (e ∸ j))·F → π

π' → F
↓j → ?
j + 1 → j
E → π
```

```input
5  3 1 4 1 5
```

```output
3
4
8
9
14
```

## Exchanging two numbers

Sorting needs to compare neighbours and exchange them. An exchange takes
both numbers out before putting either back:

```bohm
π' → A
? → n
100 → i
100 + n → e

π' → B              # Read the numbers.
(1 ∩ (e ∸ i))·C + (1 ∸ (e ∸ i))·D → π

π' → C
? → ↓i
i + 1 → i
B → π

π' → D              # One pass. w records whether any pair was exchanged.
0 → w
100 → i

π' → E
i + 1 → j
(1 ∩ (e ∸ j))·F + (1 ∸ (e ∸ j))·I → π

π' → F              # If the pair at i and j is out of order continue at G.
↓i → a
↓j → b
(1 ∩ (a ∸ b))·G + (1 ∸ (a ∸ b))·H → π

π' → G
b → ↓i
a → ↓j
1 → w

π' → H
j → i
E → π

π' → I              # Another pass if anything moved.
(1 ∩ w)·D + (1 ∸ w)·J → π

π' → J
100 → i

π' → K
(1 ∩ (e ∸ i))·L → π

π' → L
↓i → ?
i + 1 → i
K → π
```

```input
6  31 4 15 92 65 35
```

```output
4
15
31
35
65
92
```

## An index of an index

`↓` reaches one step. For two steps, take them one at a time, as the
dissertation shows:

```bohm
π' → A
200 → a             # a points at cell 200,
300 → ↓a            # which points at cell 300,
300 → p
42 → ↓p             # which holds 42.
↓a → p              # First step: p becomes 300.
↓p → ?              # Second step.
Ω → π
```

```output
42
```
