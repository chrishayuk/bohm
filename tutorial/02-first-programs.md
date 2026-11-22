# First Programs

## Reading and printing

`?` stands for the world outside the program. On the left of the arrow it is
the next number on the input tape; on the right it is the output tape.

```bohm
π' → A
? → a
a → ?
Ω → π
```

```input
42
```

```output
42
```

Read from the top:

- `π' → A` says that a group named A begins here. Every program is made of
  groups, and the first one in the file is where it starts.
- `? → a` reads a number into `a`.
- `a → ?` prints it.
- `Ω → π` stops the program.

## Formulas

Each line is a formula: an expression, an arrow, and one variable to receive
the result. The arrow is read "becomes". The dissertation writes the result
on the right because that is the order things happen in.

The expression may use `?` more than once. Each one reads another number:

```bohm
π' → A
? + ? → ?
Ω → π
```

```input
19 23
```

```output
42
```

## Names

A name is a single letter. Capital and small letters are different names, so
there are 52 of them. By custom, groups get capitals and quantities get
small letters.

Two capitals are best left alone: `S` and `X`. Chapter 4 explains why.

## Numbers in a program

A number written in a formula stands for itself:

```bohm
π' → A
? → n
2·n + 1 → ?
Ω → π
```

```input
20
```

```output
41
```

## Comments

Anything after `#` on a line is ignored. The dissertation prints an
explanation beside every group, on the left; in a file it goes on the right.

```bohm
# Seconds in a number of days.

π' → A              # Let d be given.
? → d
d·24·60·60 → ?      # Hours, minutes, seconds.
Ω → π
```

```input
7
```

```output
604800
```

## Stopping

`π` holds the group to continue at, and `Ω` means "no group". A program
stops when `π` is given `Ω`. Writing `0 → π` does the same thing.

A program that runs off the end of its last group also stops, but it is
clearer to say so.
