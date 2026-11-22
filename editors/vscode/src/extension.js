// The VS Code side of Böhm's language. The work is done by the bohm program,
// which the extension carries compiled to WebAssembly and runs with the
// editor's own Node: `bohm lsp` is the language server, and the commands run
// the same program on the file being edited.
"use strict";

const vscode = require("vscode");
const path = require("path");
const { execFile } = require("child_process");
const { LanguageClient, TransportKind } = require("vscode-languageclient/node");

/** @type {LanguageClient | undefined} */
let client;
let output;
// The numbers last given to each file, offered again next time.
const lastInput = new Map();

const custom = () => (vscode.workspace.getConfiguration("bohm").get("path") || "").trim();

async function startServer(context) {
  const program = custom();
  const builtIn = { module: context.asAbsolutePath(path.join("server", "lsp.js")), transport: TransportKind.stdio };
  const serverOptions = program ? { command: program, args: ["lsp"] } : { run: builtIn, debug: builtIn };
  client = new LanguageClient("bohm", "Böhm", serverOptions, { documentSelector: [{ language: "bohm" }] });
  try {
    await client.start();
  } catch (error) {
    client = undefined;
    vscode.window.showWarningMessage(`Böhm: the language server did not start (${error.message}). Highlighting and snippets still work.`);
  }
}

async function restartServer(context) {
  const old = client;
  client = undefined;
  if (old) await old.stop().catch(() => {});
  await startServer(context);
}

/** Runs `bohm COMMAND FILE OPTIONS` and resolves with everything it printed. */
function bohm(context, file, command, options) {
  const program = custom();
  const [exe, args, env] = program
    ? [program, [command, file, ...options], process.env]
    : [process.execPath, [context.asAbsolutePath(path.join("server", "run.js")), file, command, ...options], { ...process.env, ELECTRON_RUN_AS_NODE: "1" }];
  return new Promise((resolve) => {
    const child = execFile(exe, args, { env, timeout: 60000, maxBuffer: 16 * 1024 * 1024 }, (error, stdout, stderr) => {
      const missing = error && error.code === "ENOENT" ? `could not run "${exe}": check the bohm.path setting\n` : "";
      resolve(stdout + stderr + missing);
    });
    // Close standard input: the program must not wait there for numbers.
    child.stdin.end();
  });
}

/** Runs a command on the file in the editor and shows what it prints. */
async function run(context, command, options, { input = false } = {}) {
  const editor = vscode.window.activeTextEditor;
  if (!editor || editor.document.languageId !== "bohm") {
    vscode.window.showInformationMessage("Open a Böhm program to run it.");
    return undefined;
  }
  const document = editor.document;
  if (document.isUntitled) {
    vscode.window.showInformationMessage("Save the program first.");
    return undefined;
  }
  if (document.isDirty) await document.save();
  const file = document.uri.fsPath;
  const all = [...options];
  // A program that reads numbers is asked for them.
  if (input && /(^|[\s(\[{+·*:\/∸\-])\?\s*(→|->)/m.test(document.getText().replace(/#.*$/gm, ""))) {
    const numbers = await vscode.window.showInputBox({
      prompt: "The numbers the program reads, separated by spaces or commas",
      value: lastInput.get(file) || "",
    });
    if (numbers === undefined) return undefined;
    lastInput.set(file, numbers);
    all.push("--input", numbers.trim().split(/[\s,]+/).filter(Boolean).join(","));
  }
  output.clear();
  output.show(true);
  output.appendLine(`bohm ${[command, path.basename(file), ...all].join(" ")}`);
  const printed = await bohm(context, file, command, all);
  output.append(printed);
  return printed;
}

async function activate(context) {
  output = vscode.window.createOutputChannel("Böhm");
  const command = (name, ...how) => vscode.commands.registerCommand(name, () => run(context, ...how));
  context.subscriptions.push(
    output,
    command("bohm.run", "run", [], { input: true }),
    command("bohm.runAlgebra", "run", ["--via", "interp"], { input: true }),
    command("bohm.verify", "verify", [], { input: true }),
    command("bohm.instructions", "compile", ["--via", "bohm"]),
    vscode.commands.registerCommand("bohm.restart", () => restartServer(context)),
    vscode.workspace.onDidChangeConfiguration((event) => {
      if (event.affectsConfiguration("bohm.path")) restartServer(context);
    }),
  );
  await startServer(context);
}

function deactivate() {
  return client ? client.stop() : undefined;
}

module.exports = { activate, deactivate };
