// Runs inside VS Code, started by run-vscode.js.
const assert = require("node:assert");
const path = require("node:path");
const vscode = require("vscode");

const until = async (what, test) => {
  for (let i = 0; i < 100; i++) {
    const value = await test();
    if (value) return value;
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  throw new Error(`timed out waiting for ${what}`);
};

exports.run = async () => {
  const uri = vscode.Uri.file(path.join(vscode.workspace.workspaceFolders[0].uri.fsPath, "program.bohm"));
  const document = await vscode.workspace.openTextDocument(uri);
  assert.strictEqual(document.languageId, "bohm");
  const editor = await vscode.window.showTextDocument(document);

  // Problems arrive from the language server.
  const problems = await until("diagnostics", () => {
    const found = vscode.languages.getDiagnostics(uri);
    return found.length ? found : null;
  });
  assert.strictEqual(problems.length, 3);
  assert.ok(problems[0].message.startsWith("X is where compiled code"), problems[0].message);
  assert.strictEqual(problems[0].severity, vscode.DiagnosticSeverity.Warning);

  // Resting on a formula shows its instructions.
  const hovers = await vscode.commands.executeCommand("vscode.executeHoverProvider", uri, new vscode.Position(2, 4));
  const shown = hovers.flatMap((h) => h.contents).map((c) => c.value ?? c).join("\n");
  assert.ok(shown.includes("On the machine:") && shown.includes("501  "), shown);

  // B in `B -> pi` leads to the header of group B.
  const places = await vscode.commands.executeCommand("vscode.executeDefinitionProvider", uri, new vscode.Position(3, 0));
  assert.strictEqual((places[0].range ?? places[0].targetRange).start.line, 5);

  const outline = await vscode.commands.executeCommand("vscode.executeDocumentSymbolProvider", uri);
  assert.deepStrictEqual(outline.map((s) => s.name), ["A", "B"]);

  // The symbols come from the server and the snippets from the extension;
  // snippets are loaded a moment after the file opens.
  const labels = await until("completions", async () => {
    const offered = await vscode.commands.executeCommand("vscode.executeCompletionItemProvider", uri, new vscode.Position(4, 0));
    const found = offered.items.map((i) => i.label.label ?? i.label);
    return found.includes("→") && found.includes("group") ? found : null;
  });
  assert.ok(labels.includes("choice"));

  // Formatting writes the symbols as printed.
  await vscode.commands.executeCommand("editor.action.formatDocument");
  await until("the formatted text", () => editor.document.getText().startsWith("π' → A"));
  assert.strictEqual(editor.document.getText(), "π' → A\n? → X\nX·X + 1 → ?\nB → π\n\nπ' → B\nΩ → π\n");

  // A change that breaks the syntax becomes an error on that line.
  await editor.edit((edit) => edit.insert(new vscode.Position(4, 0), "a mod 2 + 1 → ?"));
  const broken = await until("the error", () => {
    const found = vscode.languages.getDiagnostics(uri);
    return found.length === 1 && found[0].severity === vscode.DiagnosticSeverity.Error ? found : null;
  });
  assert.strictEqual(broken[0].range.start.line, 4);
  assert.ok(broken[0].message.includes("brackets"), broken[0].message);
  await vscode.commands.executeCommand("workbench.action.files.revert");
  // The run commands, with the program the extension carries.
  const square = await vscode.workspace.openTextDocument(vscode.Uri.file(path.join(path.dirname(uri.fsPath), "square.bohm")));
  await vscode.window.showTextDocument(square);
  assert.strictEqual(await vscode.commands.executeCommand("bohm.run"), "42\n");
  assert.strictEqual(await vscode.commands.executeCommand("bohm.runAlgebra"), "42\n");
  const three = await vscode.commands.executeCommand("bohm.verify");
  assert.ok(three.endsWith("all three agree\n"), three);
  const listing = await vscode.commands.executeCommand("bohm.instructions");
  assert.ok(listing.includes("a → ?") || listing.includes("→ ?"), listing);

  // Semantic colours come from the server.
  const colours = await vscode.commands.executeCommand("vscode.provideDocumentSemanticTokens", square.uri);
  assert.ok(colours.data.length >= 5 * 10, "semantic tokens");

  console.log("extension: diagnostics, hover, definition, outline, completion, formatting, colours and the run commands verified in VS Code");
};
