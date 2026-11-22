// Checks the highlighting grammar by tokenising real lines with the same
// engine VS Code uses:  node test/grammar.test.js
const assert = require("node:assert");
const fs = require("node:fs");
const path = require("node:path");
const oniguruma = require("vscode-oniguruma");
const { Registry, parseRawGrammar } = require("vscode-textmate");

const root = path.join(__dirname, "..");
const wasm = fs.readFileSync(require.resolve("vscode-oniguruma/release/onig.wasm")).buffer;

(async () => {
  await oniguruma.loadWASM(wasm);
  const registry = new Registry({
    onigLib: Promise.resolve({
      createOnigScanner: (patterns) => new oniguruma.OnigScanner(patterns),
      createOnigString: (text) => new oniguruma.OnigString(text),
    }),
    loadGrammar: async () => {
      const file = path.join(root, "syntaxes/bohm.tmLanguage.json");
      return parseRawGrammar(fs.readFileSync(file, "utf8"), file);
    },
  });
  const grammar = await registry.loadGrammar("source.bohm");

  // The scope of each piece of a line, without the `source.bohm` every piece has.
  const scopes = (line) =>
    grammar.tokenizeLine(line, null).tokens
      .map((t) => [line.slice(t.startIndex, t.endIndex), t.scopes.slice(1).join(" ").replace(/\.bohm/g, "")])
      .filter(([text]) => text.trim());
  const expect = (line, want) => assert.deepStrictEqual(scopes(line), want, line);

  expect("π' → A              # begin", [
    ["π'", "storage.type.group"],
    ["→", "keyword.operator.assignment"],
    ["A", "entity.name.function.group"],
    ["# begin", "comment.line.number-sign"],
  ]);
  expect("pi' -> B", [
    ["pi'", "storage.type.group"],
    ["->", "keyword.operator.assignment"],
    ["B", "entity.name.function.group"],
  ]);
  expect("? → n", [
    ["?", "support.function.io"],
    ["→", "keyword.operator.assignment"],
    ["n", "variable.other"],
  ]);
  expect("(1 ∩ n)·C + (1 ∸ n)·D → π", [
    ["(", "punctuation.section.brackets"],
    ["1", "constant.numeric"],
    ["∩", "keyword.operator.arithmetic"],
    ["n", "variable.other"],
    [")", "punctuation.section.brackets"],
    ["·", "keyword.operator.arithmetic"],
    ["C", "entity.name.function.group"],
    ["+", "keyword.operator.arithmetic"],
    ["(", "punctuation.section.brackets"],
    ["1", "constant.numeric"],
    ["∸", "keyword.operator.arithmetic"],
    ["n", "variable.other"],
    [")", "punctuation.section.brackets"],
    ["·", "keyword.operator.arithmetic"],
    ["D", "entity.name.function.group"],
    ["→", "keyword.control.jump"],
    ["π", "keyword.control.jump"],
  ]);
  expect("Omega -> pi", [
    ["Omega", "constant.language.omega"],
    ["->", "keyword.control.jump"],
    ["pi", "keyword.control.jump"],
  ]);
  expect("a mod 10 → ↓p", [
    ["a", "variable.other"],
    ["mod", "keyword.operator.word"],
    ["10", "constant.numeric"],
    ["→", "keyword.operator.assignment"],
    ["↓", "keyword.operator.indirection"],
    ["p", "variable.other"],
  ]);
  // S and X stand out: compiled code uses them.
  expect("X·m3 → S", [
    ["X", "invalid.deprecated.reserved"],
    ["·", "keyword.operator.arithmetic"],
    ["m3", "variable.other"],
    ["→", "keyword.operator.assignment"],
    ["S", "invalid.deprecated.reserved"],
  ]);
  expect("const m = 850", [
    ["const", "keyword.other.directive"],
    ["m", "variable.other.constant"],
    ["=", "keyword.operator.assignment"],
    ["850", "constant.numeric"],
  ]);
  expect("origin 200", [
    ["origin", "keyword.other.directive"],
    ["200", "constant.numeric"],
  ]);
  // π′ in the middle of a formula is not a jump.
  expect("π' + 1 → a", [
    ["π'", "variable.language.pi-prime"],
    ["+", "keyword.operator.arithmetic"],
    ["1", "constant.numeric"],
    ["→", "keyword.operator.assignment"],
    ["a", "variable.other"],
  ]);

  // Every example and every printed program: nothing left without a scope.
  let lines = 0;
  for (const dir of ["examples", "programs"]) {
    const folder = path.join(root, "../..", dir);
    for (const file of fs.readdirSync(folder).filter((f) => f.endsWith(".bohm"))) {
      for (const line of fs.readFileSync(path.join(folder, file), "utf8").split("\n")) {
        lines++;
        for (const [text, scope] of scopes(line)) {
          assert.ok(scope, `${file}: no scope for "${text}" in: ${line}`);
        }
      }
    }
  }
  console.log(`grammar: 10 lines checked piece by piece, ${lines} lines of real programs fully scoped`);
})().catch((error) => {
  console.error(error.message);
  process.exit(1);
});
