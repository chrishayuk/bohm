# Independent review

## Who reviewed it

The repository was reviewed on 22 November 2026 by OpenAI Codex CLI
(codex-cli 0.166.0), run as an independent second model. It is not a
person, and it had not seen how the repository was written. It was given
the repository and the three sources (the French printing of 1954,
Sestoft's 2016 translation, and Knuth and Trabb Pardo's 1976 report) and
asked to check the code, the transcriptions and the historical claims.

Its findings were then checked one by one, against the code, by running
the tools, and against the page images, before anything was changed. This
file records what it found and what was done. Commit hashes refer to this
repository's `main` branch.

## What it checked

* The build and every check the README describes: `cargo build --release`,
  `cargo test` (27 tests at the time, all passing), the tutorial checker
  (64 blocks, none failing), the playground build and its test (42
  examples through WebAssembly, none failing), the dataset builder (79
  entries, byte for byte the committed file) and the two `bohm check`
  counts. The VS Code extension's grammar and editor tests could not run
  for it, because it had no network to install their dependencies; they
  have since been run here and pass.
* The three readings against each other on formulas the tests do not
  generate.
* Each erratum against the page it rests on, and whether it is needed.
* A spot check of the transcriptions: 24 passages compared with the page
  images character by character, with spacing, subscripts, `π′`/`pi'` and
  written-out powers of ten treated as conventions. 21 matched. Three did
  not: group J′ of the compiler, the boundary of groups E and F in Euclid
  for n numbers, and the loader's group β.

## The findings, and what was done

**1. The algebraic reading and the machine disagreed on formulas that read
`?` more than once.** Accepted. With input 1, 2, 3, `(?+(?·?)) → ?`
printed 7 as algebra and 5 on the machine: the interpreter evaluated left
to right, while chapter 5 writes the instructions of a bracketed formula in
decreasing order of region (5.32), so the inner multiplication reads the
tape first. The random test of meaning had left `?` out of its operands.
The thesis does not say what order it means. Since the machine's order is
the one the original method produces, the interpreter now follows it, and
the rule is stated as ours in the README, in `src/interp.rs` and in
tutorial chapter 4. Fixed in 05aca0d; tests
`input_is_read_in_the_order_of_the_instructions`, and `?` added to
`random_formulas_mean_the_same`, both of which fail without the fix.

**2. Calling `names` a fourth necessary slip overstated what the tests
show.** Accepted. Reproduced: with every formula of chapter 7 put in
brackets, and only `branch`, `store` and `link` applied, the compiler
compiles itself and keeps its X. `names` is needed only because the
compiler's formulas without brackets are compiled as chapter 6
polynomials, which is how Knuth and Trabb Pardo describe Böhm's compiler
treating a formula that does not begin with a bracket, but it is still a
reading. It is now labelled an interpretation in `bohm errata`, the
README and tutorial chapter 13, and the claim that self-compilation fails
without any one correction is stated with that condition. 7834ccb; test
`names_is_needed_only_for_polynomials`.

**3. The compiler transcription silently supplied two arrows.** Accepted.
Page 43 prints group J′ as `j·m₂ + h·m₃ + g₄   ?` and `B′   π`, with no
arrows, while the file said its formulas were "left exactly as printed".
The arrows stay, since a formula cannot be read without one, but the file's
header, the README and `bohm errata` now say so and call it a slip, and tutorial
chapter 13 says that `--errata none` gives the text as printed apart from
those two arrows.
7834ccb.

**4. Euclid for n numbers had an undisclosed correction and an added
instruction.** Accepted. Page 26 heads the group described as F with a
second `π′ → E`; the file had written F without saying so. It also had a
jump `F → π` at the end of group E that the printing does not have. The
jump is removed (E falls through into F, as printed, and the program still
agrees three ways), and the header correction is disclosed as a slip in
the file, the README and `bohm errata`. dc30ba7; test
`euclid_for_n_numbers_group_e_as_printed`.

**5. Tutorial chapter 6 said "Nobody noticed for as long as the program
existed only on paper".** Accepted. Nothing here establishes that. The
sentence now says we have not found the slip reported in print, and that
Sestoft ran the compiler without saying which corrections he needed.
f32a85c.

**6. Smaller documentation claims.** Accepted, all three.
"The language has no way to load a number" is replaced: section 2.6 makes
a numeral the address of a cell holding it, and what the thesis lacks is a
way of filling those cells. "Every example runs twice" now names the
example run on the machine only. "Compared in every test" now says which
tests compare the two translators. f32a85c and 7834ccb.

**7. The remaining errata mostly hold.** Accepted as the review states it.
`branch` and `store` are slips. `link` is a slip, but the fix is not shown
to be the smallest (Knuth and Trabb Pardo thought one more statement would
do; this one changes one and adds two), and `bohm errata` and the README
now say so. The ineffective syntax checks, TPK's U/V label, the missing
`0 → S` in table (6.2) and the English translation's missing group W were
confirmed. The review also pointed out a conflict not recorded before:
group F′ is `h·m₃ + g₆ → ?` in the French and `h·m₂ + g₆ → ?` in the
English. That, the table (6.2) case and W are now listed as conflicts in
the README and in `bohm errata`. 7834ccb, f32a85c.

**8. The other additions are reasonable, but their status should be
explicit.** Accepted. Each item under "What is not in the thesis" now
carries its kind. Completing brackets and the values of g2..g6 are
labelled interpretations; the constants prefix, the decision table as a
group, the cells for extra names, the loader's placement, the order of
reading `?`, the handling of undefined cases and the alternating bases for
self-compilation are labelled ours. The README now says that the
period-two fixed point describes this arrangement of memory. f32a85c.

**9. Transcription spot check.** Accepted, with one addition. Besides
findings 3 and 4, the review found that the loader writes `π′` where the
printing has `π″` (footnote 20 says this is only for clarity) and adds
brackets around `a∸b` and `c + 9490010000`, while its header said "two
things differ". All of these are now listed in `programs/loader.bohm`. In
re-checking page 28 for this, a further silent change came to light that
the review had not reported: the printing gives the constant in group β as
94999010000, which recognises no group header (a header plus that constant
is 5509000000 modulo 10¹⁰, not 0). The loader uses 9490010000, as
Sestoft's translation does. This is now disclosed as a slip in the loader,
the README and `bohm errata`, and a test shows the printed constant fails.
7834ccb; test `loader_recognises_group_headers`.

**10. Build and checks.** All passed for the review except the extension
tests it could not install. After the changes above: 30 Rust tests pass,
the tutorial checker reports 65 blocks and none failing, the playground
test 42 examples and none failing, the dataset builder writes 80 entries,
`bohm selfhost` reproduces itself, and the extension's grammar, server and
VS Code tests pass after `npm ci`.

## Disputed

Nothing was disputed outright. Two points were taken further than the
review put them: `names` is labelled an interpretation rather than a
consequence of our choices, because compiling unbracketed formulas as
polynomials is the method the thesis gives, not one we chose; and the
loader's constant, which the review passed over, is disclosed as above.
