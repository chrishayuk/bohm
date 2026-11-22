# Böhm for VS Code

Editor support for the language of Corrado Böhm's 1951 dissertation.

- **Highlighting.** Group headers, jumps (`… → π`), input and output (`?`),
  operations and numbers each have their own colour. `S` and `X` are marked,
  because compiled code uses them.
- **The symbols a keyboard lacks.** Type `->`, `pi`, `Omega`, `-`, `*`, `/`,
  `~`, `max`, `min` and `@`. Saving the file writes them as `→ π Ω ∸ · : ÷ ∪ ∩
  ↓`. Completion offers the symbols too, and there are snippets for a group,
  a choice, a loop, a subroutine call and a stack.
- **Problems as you type.** Syntax errors on the line they belong to, and
  warnings for what compiles but will not work: using `S` or `X` beside a
  formula without brackets, a jump to a group that does not exist, and a
  group that begins twice.
- **The machine behind a formula.** Rest the pointer on a formula to see the
  instruction numbers it becomes, with their addresses. Rest it on a symbol
  to see what it means and which cell it is.
- **Groups.** Go to a group's definition, find where it is used, rename it
  everywhere, fold it, and see the groups of a program in the outline.
- **Run it.** Cmd+Enter (Ctrl+Enter) runs the program on Böhm's machine.
  The command palette also has *Run as Algebra*, *Run All Three Ways and
  Compare* and *Show the Instructions*.

## Install

Download `bohm-language-0.1.1.vsix` from the
[latest release](https://github.com/chrishayuk/bohm/releases/latest) and
install it:

```
code --install-extension bohm-language-0.1.1.vsix
```

That is all it needs. The extension carries the `bohm` program compiled to
WebAssembly and runs it with the editor's own Node. To use a program you
built yourself, set `bohm.path` in the settings.

Saving a file rewrites ASCII spellings as symbols. To turn that off:

```json
"[bohm]": { "editor.formatOnSave": false }
```

## Build it yourself

```
cd editors/vscode
npm install
npm run package          # builds the WebAssembly, bundles, writes the .vsix
```

## How it works

`bohm lsp` is a language server, written in Rust with the rest of the tools
and with no dependencies. `server/lsp.js` runs it from `server/bohm.wasm`,
and `server/run.js` runs the other commands the same way. Any editor that
speaks the Language Server Protocol can use the server; see
[`../README.md`](../README.md).

## Tests

```
npm test                 # the grammar, and the two server scripts under Node
npm run test:vscode      # the whole extension inside a real VS Code
```

The second downloads a copy of VS Code into `.vscode-test/` the first time.
It opens a program and checks the problems, hover, go to definition,
outline, completion, formatting, colours and each run command.
