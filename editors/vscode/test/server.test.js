// Checks the two scripts the extension starts, with plain Node:
// the language server and the runner.   node test/server.test.js
const assert = require("node:assert");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const { spawn, spawnSync } = require("node:child_process");

const server = path.join(__dirname, "../server");
const frame = (message) => {
  const body = Buffer.from(JSON.stringify({ jsonrpc: "2.0", ...message }));
  return Buffer.concat([Buffer.from(`Content-Length: ${body.length}\r\n\r\n`), body]);
};

(async () => {
  // The runner: a file, a command, and numbers for the program to read.
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "bohm-server-"));
  const file = path.join(dir, "gcd.bohm");
  fs.copyFileSync(path.join(__dirname, "../../../programs/euclid.bohm"), file);
  const ran = spawnSync(process.execPath, [path.join(server, "run.js"), file, "run", "--input", "1071,462"], { input: "", encoding: "utf8" });
  assert.strictEqual(ran.stdout, "21\n", ran.stderr);
  assert.strictEqual(ran.status, 0);
  const missing = spawnSync(process.execPath, [path.join(server, "run.js"), path.join(dir, "none.bohm"), "run"], { input: "", encoding: "utf8" });
  assert.strictEqual(missing.status, 2);

  // The server: a conversation over its standard input and output, with
  // the answers read as they arrive, as an editor does.
  const child = spawn(process.execPath, [path.join(server, "lsp.js")]);
  let buffer = Buffer.alloc(0);
  const waiting = [];
  const messages = [];
  child.stdout.on("data", (chunk) => {
    buffer = Buffer.concat([buffer, chunk]);
    for (;;) {
      const split = buffer.indexOf("\r\n\r\n");
      if (split < 0) return;
      const length = Number(/Content-Length: (\d+)/.exec(buffer.subarray(0, split).toString())[1]);
      if (buffer.length < split + 4 + length) return;
      const message = JSON.parse(buffer.subarray(split + 4, split + 4 + length).toString());
      buffer = buffer.subarray(split + 4 + length);
      const next = waiting.shift();
      if (next) next(message);
      else messages.push(message);
    }
  });
  const read = () => new Promise((resolve) => (messages.length ? resolve(messages.shift()) : waiting.push(resolve)));
  const uri = "file:///program.bohm";

  child.stdin.write(frame({ id: 1, method: "initialize", params: { capabilities: {} } }));
  const hello = await read();
  assert.strictEqual(hello.result.serverInfo.name, "bohm");
  assert.deepStrictEqual(hello.result.capabilities.semanticTokensProvider.legend.tokenTypes.slice(0, 3), ["comment", "string", "number"]);

  child.stdin.write(frame({ method: "textDocument/didOpen", params: { textDocument: { uri, text: "π' → A\n? → X\nX·X + 1 → ?\n" } } }));
  const published = await read();
  assert.strictEqual(published.params.diagnostics.length, 3);

  child.stdin.write(frame({ id: 2, method: "textDocument/hover", params: { textDocument: { uri }, position: { line: 2, character: 4 } } }));
  const hover = await read();
  assert.ok(hover.result.contents.value.includes("On the machine:"));

  child.stdin.write(frame({ id: 3, method: "shutdown" }));
  await read();
  child.stdin.write(frame({ method: "exit" }));
  const code = await new Promise((resolve) => child.on("exit", resolve));
  assert.strictEqual(code, 0);
  console.log("server: the runner and a live language-server conversation verified under Node");
})().catch((error) => {
  console.error(error.message || error);
  process.exit(1);
});
