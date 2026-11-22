# Böhm's language in your editor

Two pieces give an editor an understanding of the notation:

- **`bohm lsp`**, a language server. It is a command of the `bohm` program
  itself, so there is nothing else to build, and it works with any editor
  that speaks the Language Server Protocol.
- **A Visual Studio Code extension** in [`vscode`](vscode), which carries
  the program and adds syntax highlighting, snippets and run commands. Its
  [README](vscode/README.md) lists the features.

Both are built on [`src/check.rs`](../src/check.rs), the same code behind
`bohm check`.

## The language server

```sh
cargo install --git https://github.com/chrishayuk/bohm
```

or, in a clone of this repository, `cargo build --release`. The server is
then `bohm lsp`. It talks on standard input and output.

It provides diagnostics, hover, go to definition, references, document
highlights, rename, completion, document symbols, folding ranges,
formatting and semantic tokens. Semantic tokens mean an editor with no
grammar for the notation still gets highlighting. Formatting writes `->`,
`pi`, `Omega`, `-`, `*` and the other keyboard spellings as `→ π Ω ∸ ·`.

### Neovim (0.11 or later)

```lua
vim.filetype.add({ extension = { bohm = "bohm" } })
vim.lsp.config("bohm", {
  cmd = { "/path/to/bohm", "lsp" },
  filetypes = { "bohm" },
})
vim.lsp.enable("bohm")
```

### Helix

In `languages.toml`:

```toml
[language-server.bohm-lsp]
command = "/path/to/bohm"
args = ["lsp"]

[[language]]
name = "bohm"
scope = "source.bohm"
file-types = ["bohm"]
comment-token = "#"
language-servers = ["bohm-lsp"]
```

These two configurations follow each editor's documented format but have
not been run here. The server itself is tested over standard input and
output by `tests/lsp.rs` and `vscode/test/server.test.js`.

## Checking a program from the command line

```sh
bohm check program.bohm
```

prints one line per problem, as `file:line:column: kind: message`, and
exits with status 1 if any is an error. `bohm format program.bohm` prints
the program with its symbols.

## In a web page

[`../web/lsp-worker.js`](../web/lsp-worker.js) runs the same server in a
browser worker, from the same `bohm.wasm` the playground uses.
