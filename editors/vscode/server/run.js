// Runs one bohm command on a file:  node run.js FILE run --via interp --input 3,4
// The program compiled for WASI has no file system here, so the file is read
// by this script and handed over on standard input, where `-` stands for it.
"use strict";

const fs = require("fs");
const path = require("path");
const { run } = require("./wasi.js");

const [file, command, ...rest] = process.argv.slice(2);
if (!file || !command) {
  console.error("usage: run.js FILE COMMAND [OPTIONS]");
  process.exit(2);
}
let program;
try {
  program = fs.readFileSync(file, "utf8");
} catch (error) {
  console.error(`bohm: ${file}: ${error.message}`);
  process.exit(2);
}
// After the program comes a line of four dashes and then the numbers it
// reads, which arrive with --input.
process.exitCode = run(path.join(__dirname, "bohm.wasm"), {
  args: [command, "-", ...rest],
  prefix: Buffer.from(program + "\n----\n"),
});
