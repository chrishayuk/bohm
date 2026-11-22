// Runs the extension inside a real VS Code and drives it:  node test/run-vscode.js
// The first run downloads a copy of VS Code into .vscode-test/.
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const { runTests } = require("@vscode/test-electron");

(async () => {
  const root = path.join(__dirname, "..");
  if (!fs.existsSync(path.join(root, "dist/extension.js"))) throw new Error("build the extension first: npm run build");
  // No settings: the extension must work with the program it carries.
  const workspace = fs.mkdtempSync(path.join(os.tmpdir(), "bohm-vscode-"));
  fs.writeFileSync(path.join(workspace, "square.bohm"), "π' → A\n6 → a\na·a + 6 → ?\nΩ → π\n");
  fs.writeFileSync(
    path.join(workspace, "program.bohm"),
    "pi' -> A\n? -> X\nX*X + 1 -> ?\nB -> pi\n\npi' -> B\nOmega -> pi\n",
  );
  await runTests({
    extensionDevelopmentPath: root,
    extensionTestsPath: path.join(__dirname, "suite.js"),
    launchArgs: [workspace, "--disable-extensions", "--disable-workspace-trust"],
  });
})().catch((error) => {
  console.error(error.message || error);
  process.exit(1);
});
