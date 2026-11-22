# Formulas

You can write a formula in whatever way reads best. It helps, though, to
know what becomes of it, because Böhm's compiler accepts only two shapes.

## The two shapes

**A polynomial** has no brackets at all and uses only `+`, `∸`, `·` and `:`.

```text
3·n + 1 → n
a:m·b → ?
```

**A bracketed formula** begins with a bracket and has a pair of brackets
around every single operation.

```text
(a + b)·c ∸ d → x           as you might write it
(((a + b)·c) ∸ d) → x       as the compiler needs it
```

The dissertation itself does not keep to this. Böhm writes
`{[(1 ∸ r)·C] + [(1 ∩ r)·D]} → π` and asks the reader, in a footnote, to
pardon him for the sake of clarity. So the tools do what a careful typist
would have done: any formula that is not a polynomial has the missing
brackets put in before it goes to the compiler. `--emit normal` shows the
result:

```console
$ bohm compile examples/collatz.bohm --emit normal
π′ → A
? → n
π′ → B
n → ?
((‹1›∩(n∸‹1›))·C) → π
π′ → C
(n mod ‹2›) → r
(((‹1›∸r)·D)+(r·E)) → π
π′ → D
n:‹2› → n
B → π
π′ → E
‹3›·n+‹1› → n
B → π
```

Three things happened there. `n mod 2 → r` got brackets, because `mod` is
not one of the four polynomial operations. The choice at the end of group C
got a bracket round every operation. And `3·n + 1 → n` was left alone: it is
a polynomial already.

Numbers appear as `‹2›` and `‹3›` because, to the compiler, a number in a
formula is a variable like any other: a cell that happens to hold 2.

## Why S and X are taken

The compiler works out a polynomial using two variables of its own. It
keeps the running sum in `S` and the product it is building in `X`. Here is
what it makes of a single formula:

```console
$ bohm compile examples/add.bohm
# instructions           500..503
     0000 05 0999 0028   π′ → A
500  0000 05 0001 0046   ? → S
501  0046 01 0001 0046   S+? → S
502  0000 05 0046 0001   S → ?
503  0000 05 0998 0000   Ω → π
```

Those are real variables, the same `S` and `X` you could write yourself. So
a program that keeps something in `X` loses it at the next polynomial with
a `·` or `:` in it:

```bohm
# machine only
π' → A
7 → X
2·3 + 1 → y
X → ?               # prints 6, not 7
Ω → π
```

```output
6
```

The rule is simple: **do not name anything S or X**. Böhm says which two
letters his method uses, but never warns the programmer off them, and his
own compiler uses `X` for something else (chapter 13).

A bracketed formula does not have the problem. Its intermediate results go
to cells that have no letter.

## Which shape to use

For a program's meaning it makes almost no difference, and you can let the
tools choose. There are two exceptions. A polynomial works in `S` and `X`,
as above. And a formula that reads `?` more than once reads the tape in the
order of its instructions: a polynomial from left to right, a bracketed
formula innermost bracket first, the last bracket opened before the ones
opened earlier. The dissertation does not say what order it means; the
tools follow the machine, as algebra too.

```bohm
π' → A
?+?·? → ?             # 1 + 2·3
(?+(?·?)) → ?         # the multiplication first: 4·5, then 6 + 20
Ω → π
```

```input
1 2 3 4 5 6
```

```output
7
26
```

If you care how many instructions the machine executes:

| Formula | Instructions |
|---------|--------------|
| `a → b` | 1 |
| `(a + b) → c` | 2 |
| `a + b → c` | 3 |
| `(a·b) → c` | 2 |
| `a·b → c` | 5 |

A bracketed formula costs one instruction per operation and one more to
store the result. A polynomial costs a little more, because it always goes
through `S`.

## One result per formula

The right of the arrow is always one variable. There is no way to assign to
two things at once, and no way to use the result of a formula inside
another: give it a name and use the name.
