// Builds the files the playground needs:  node web/build.mjs
//   web/bohm.wasm      the bohm program, compiled for WASI
//   web/examples.json  the example programs, with their input
import { execSync } from "node:child_process";
import { copyFileSync, existsSync, readFileSync, readdirSync, writeFileSync } from "node:fs";

const root = new URL("..", import.meta.url).pathname;
execSync("cargo build --release --target wasm32-wasip1", { cwd: root, stdio: "inherit" });
copyFileSync(`${root}target/wasm32-wasip1/release/bohm.wasm`, `${root}web/bohm.wasm`);

// Programs from the literature, shown under these names with this input.
const printed = [
  { name: "thesis-euclid", file: "programs/euclid.bohm", input: "1071 462\n" },
  { name: "thesis-euclid-n", file: "programs/euclid-n.bohm", input: "4  1071 462 84 147\n" },
  { name: "knuth-tpk", file: "programs/tpk.bohm", input: "0 1 2 3 4 5 6 7 8 9 10\n" },
  { name: "knuth-tpk-renamed", file: "programs/tpk-renamed.bohm", input: "0 1 2 3 4 5 6 7 8 9 10\n" },
];

// Shown first, in this order; the rest follow alphabetically.
const first = ["echo", "add", "arithmetic", "countdown", "compare", "sum-to-n", "factorial",
  "fibonacci", "thesis-euclid", "collatz", "primes", "bubble-sort", "jump-table", "hanoi",
  "self-modifying", "knuth-tpk", "knuth-tpk-renamed"];

const examples = readdirSync(`${root}examples`).filter((f) => f.endsWith(".bohm")).map((f) => {
  const name = f.slice(0, -5);
  const input = `${root}examples/${name}.in`;
  return {
    name,
    program: readFileSync(`${root}examples/${f}`, "utf8"),
    input: existsSync(input) ? readFileSync(input, "utf8") : "",
  };
});
for (const p of printed) {
  examples.push({ name: p.name, program: readFileSync(root + p.file, "utf8"), input: p.input });
}
examples.sort((a, b) => {
  const [i, j] = [first.indexOf(a.name), first.indexOf(b.name)];
  if (i !== -1 || j !== -1) return (i === -1 ? 99 : i) - (j === -1 ? 99 : j);
  return a.name.localeCompare(b.name);
});
for (const example of examples) {
  // A program that overwrites its own instructions has no meaning as algebra.
  example.machineOnly = example.program.startsWith("# machine only");
}
writeFileSync(`${root}web/examples.json`, JSON.stringify(examples));
console.log(`web/bohm.wasm and web/examples.json (${examples.length} programs) written`);
