// Runs every example through the WebAssembly build and the browser's WASI
// host, and compares the output with the .out file.   node web/test.mjs
import { readFileSync, readdirSync, existsSync } from "node:fs";
import { run, stdinFor } from "./wasi.js";
import { ask } from "./lsp.js";

const root = new URL("..", import.meta.url).pathname;
const module = new WebAssembly.Module(readFileSync(root + "web/bohm.wasm"));

function bohm(args, program = "", input = "") {
  let out = "";
  let err = "";
  const decoder = new TextDecoder();
  const code = run(module, stdinFor(program, input), (fd, bytes) => {
    const text = decoder.decode(bytes, { stream: true });
    if (fd === 1) out += text;
    else err += text;
  }, args);
  return { code, out, err };
}

let checked = 0;
let failed = 0;
const fail = (what) => {
  failed++;
  console.log(`FAIL ${what}`);
};

for (const file of readdirSync(root + "examples").filter((f) => f.endsWith(".bohm")).sort()) {
  const name = file.slice(0, -5);
  const program = readFileSync(`${root}examples/${file}`, "utf8");
  const inFile = `${root}examples/${name}.in`;
  const input = existsSync(inFile) ? readFileSync(inFile, "utf8") : "";
  const expected = readFileSync(`${root}examples/${name}.out`, "utf8");
  const ways = program.startsWith("# machine only") ? [[]] : [[], ["--via", "interp"]];
  for (const way of ways) {
    const { code, out } = bohm(["run", "-", ...way], program, input);
    if (code !== 0 || out !== expected) fail(`${name} ${way.join(" ")} (exit ${code})`);
  }
  checked++;
}

// The examples in the published list all exist and carry their input.
const listed = JSON.parse(readFileSync(root + "web/examples.json", "utf8"));
if (listed.length !== checked + 4) fail(`examples.json lists ${listed.length} programs`);

// An error comes back as a message and a non-zero exit code.
const refused = bohm(["run", "-"], "π' → A\nn mod 2 + 1 → ?\n");
if (refused.code !== 1 || !refused.err.includes("put brackets around")) fail("error reporting");

// The other commands the playground offers.
const listing = bohm(["compile", "-", "--via", "bohm"], "π' → A\n? + ? → ?\nΩ → π\n");
if (listing.code !== 0 || !listing.out.includes("S+? → S")) fail("compile");
const three = bohm(["verify", "-"], listed.find((e) => e.name === "thesis-euclid").program, "1071 462");
if (three.code !== 0 || !three.out.endsWith("all three agree\n")) fail("verify");
const printed = bohm(["compile", "-", "--via", "bohm", "--errata", "none"], "π' → A\n? → a\na → ?\nΩ → π\n");
if (!printed.out.includes("(not an instruction)")) fail("the compiler as printed");
const self = bohm(["selfhost"]);
if (self.code !== 0 || !self.out.endsWith("the compiler reproduces itself\n")) fail("selfhost");

// The language server, as the page calls it.
const tpk = listed.find((e) => e.name === "knuth-tpk").program;
const answer = ask(module, tpk, [
  { method: "textDocument/semanticTokens/full" },
  { method: "textDocument/hover", params: { position: { line: tpk.split("\n").findIndex((l) => l.includes("→ π")), character: 0 } } },
]);
if (answer.diagnostics.length !== 5 || !answer.diagnostics[0].message.startsWith("X is where")) fail("language server: problems");
if (!(answer.results[0].data.length > 100) || answer.results[0].data.length % 5) fail("language server: colours");
if (!answer.results[1].contents.value.includes("On the machine:") || !answer.results[1].range) fail("language server: hover");
const clean = ask(module, "π' → A\n? → a\na → ?\n", [{ method: "textDocument/completion", params: { position: { line: 1, character: 0 } } }]);
if (clean.diagnostics.length !== 0 || !clean.results[0].some((item) => item.label === "→")) fail("language server: completion");

console.log(`${checked} examples checked through WebAssembly, ${failed} failed`);
process.exit(failed ? 1 : 0);
