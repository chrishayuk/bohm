# Groups and Jumps

## Groups

A program is cut into groups. A group is a run of formulas that is entered
at the top and left at the bottom. It begins with a line that names it:

```text
π' → B
```

`π′` means "the group that begins here", so the line reads "the group that
begins here is called B". It does nothing when the program runs. It is a
label.

## Continuing somewhere else

`π` means "the group to continue at". Assigning a group to it is a jump:

```bohm
π' → A
1 → ?
C → π

π' → B
2 → ?
Ω → π

π' → C
3 → ?
B → π
```

```output
1
3
2
```

The program starts at the first group in the file. After that the order of
the groups does not matter, which Böhm counted as an advantage: a
description stays intelligible however its parts are shuffled.

## Falling through

A group that does not end by assigning to `π` simply runs on into the group
written after it:

```bohm
π' → A
1 → ?

π' → B
2 → ?
Ω → π
```

```output
1
2
```

Böhm usually ends every group with a jump, even to the next one.

## A loop

A jump backwards repeats. This program would count for ever, so it needs a
way out, and the way out is the subject of the next chapter. For now, read
the last line of group B as "continue at C unless n is 0":

```bohm
π' → A
? → n

π' → B
n → ?
(1 ∩ n)·C → π

π' → C
n ∸ 1 → n
B → π
```

```input
5
```

```output
5
4
3
2
1
0
```

## A group name is a number

`B` is a variable like any other. What it holds is the place where group B
begins. That means a group can be kept in another variable and used later:

```bohm
π' → A
C → z               # remember C
B → π

π' → B
1 → ?
z → π               # continue wherever z says

π' → C
2 → ?
Ω → π
```

```output
1
2
```

Chapter 9 uses this to write subroutines. It also means you must not use
the same letter for a group and for a quantity.
