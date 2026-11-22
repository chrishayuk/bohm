//! Talks to `bohm lsp` as an editor would, over its standard input and output.

use bohm::json::{self, obj, Json};
use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

struct Client {
    child: Child,
    to: ChildStdin,
    from: BufReader<ChildStdout>,
    next: usize,
}

const URI: &str = "file:///tmp/program.bohm";

impl Client {
    fn start() -> Client {
        let mut child = Command::new(env!("CARGO_BIN_EXE_bohm"))
            .arg("lsp")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        let to = child.stdin.take().unwrap();
        let from = BufReader::new(child.stdout.take().unwrap());
        Client { child, to, from, next: 1 }
    }

    fn send(&mut self, message: Json) {
        let mut body = String::new();
        message.write(&mut body);
        write!(self.to, "Content-Length: {}\r\n\r\n{}", body.len(), body).unwrap();
        self.to.flush().unwrap();
    }

    fn read(&mut self) -> Json {
        let mut length = 0;
        loop {
            let mut line = String::new();
            assert!(self.from.read_line(&mut line).unwrap() > 0, "the server closed its output");
            match line.trim_end().split_once(": ") {
                Some(("Content-Length", n)) => length = n.parse().unwrap(),
                _ if line.trim_end().is_empty() => break,
                _ => {}
            }
        }
        let mut body = vec![0; length];
        self.from.read_exact(&mut body).unwrap();
        json::parse(std::str::from_utf8(&body).unwrap()).unwrap()
    }

    fn notify(&mut self, method: &str, params: Json) {
        self.send(obj([("jsonrpc", "2.0".into()), ("method", method.into()), ("params", params)]));
    }

    fn request(&mut self, method: &str, params: Json) -> Json {
        let id = self.next;
        self.next += 1;
        self.send(obj([
            ("jsonrpc", "2.0".into()),
            ("id", id.into()),
            ("method", method.into()),
            ("params", params),
        ]));
        let reply = self.read();
        assert_eq!(reply.get("id").num(), Some(id), "{:?}", reply);
        reply
    }

    fn at(line: usize, character: usize) -> Json {
        obj([
            ("textDocument", obj([("uri", URI.into())])),
            ("position", obj([("line", line.into()), ("character", character.into())])),
        ])
    }
}

#[test]
fn an_editor_session() {
    let mut c = Client::start();
    let init = c.request("initialize", obj([("capabilities", obj([]))]));
    let caps = init.get("result").get("capabilities");
    assert_eq!(caps.get("hoverProvider"), &Json::Bool(true));
    assert_eq!(caps.get("textDocumentSync").num(), Some(1));
    c.notify("initialized", obj([]));

    // Opening a file reports its problems: here, the use of X beside a
    // formula without brackets.
    let text = "pi' -> A\n? -> X\nX*X + 1 -> ?\nB -> pi\n\npi' -> B\nOmega -> pi\n";
    c.notify(
        "textDocument/didOpen",
        obj([("textDocument", obj([("uri", URI.into()), ("text", text.into())]))]),
    );
    let published = c.read();
    assert_eq!(published.get("method").str(), Some("textDocument/publishDiagnostics"));
    let problems = published.get("params").get("diagnostics").items();
    assert_eq!(problems.len(), 3, "{:?}", problems);
    assert_eq!(problems[0].get("severity").num(), Some(2));
    assert_eq!(problems[0].get("range").get("start").get("line").num(), Some(1));
    assert!(problems[0].get("message").str().unwrap().starts_with("X is where compiled code"));

    // Resting on a formula shows its instructions.
    let hover = c.request("textDocument/hover", Client::at(2, 4));
    let shown = hover.get("result").get("contents").get("value").str().unwrap();
    assert!(shown.contains("On the machine:") && shown.contains("501  "), "{}", shown);

    // B in `B -> pi` leads to the header of group B.
    let def = c.request("textDocument/definition", Client::at(3, 0));
    assert_eq!(def.get("result").get("range").get("start").get("line").num(), Some(5));
    let refs = c.request("textDocument/references", Client::at(3, 0));
    assert_eq!(refs.get("result").items().len(), 2);

    // Renaming a group changes its header and every use.
    let mut rename = Client::at(3, 0);
    if let Json::Obj(fields) = &mut rename {
        fields.push(("newName".into(), "K".into()));
    }
    let renamed = c.request("textDocument/rename", rename);
    assert_eq!(renamed.get("result").get("changes").get(URI).items().len(), 2);
    let marked = c.request("textDocument/documentHighlight", Client::at(3, 0));
    assert_eq!(marked.get("result").items().len(), 2);
    let folds = c.request("textDocument/foldingRange", Client::at(0, 0));
    assert_eq!(folds.get("result").items()[0].get("endLine").num(), Some(3));
    let colours = c.request("textDocument/semanticTokens/full", Client::at(0, 0));
    // `pi'` at the start of line 0 is a keyword (7), three units long.
    let data: Vec<usize> =
        colours.get("result").get("data").items().iter().map(|n| n.num().unwrap()).collect();
    assert_eq!(&data[..5], [0, 0, 3, 7, 0]);
    assert_eq!(hover.get("result").get("range").get("start").get("character").num(), Some(4));

    let outline = c.request("textDocument/documentSymbol", Client::at(0, 0));
    let names: Vec<&str> =
        outline.get("result").items().iter().map(|s| s.get("name").str().unwrap()).collect();
    assert_eq!(names, ["A", "B"]);

    let offered = c.request("textDocument/completion", Client::at(0, 0));
    assert!(offered.get("result").items().iter().any(|i| i.get("label").str() == Some("→")));

    // Formatting writes the symbols as printed.
    let edits = c.request("textDocument/formatting", Client::at(0, 0));
    let new = edits.get("result").items()[0].get("newText").str().unwrap();
    assert_eq!(new, "π' → A\n? → X\nX·X + 1 → ?\nB → π\n\nπ' → B\nΩ → π\n");

    // A change that breaks the syntax reports an error on that line.
    c.notify(
        "textDocument/didChange",
        obj([
            ("textDocument", obj([("uri", URI.into())])),
            ("contentChanges", vec![obj([("text", "π' → A\na mod 2 + 1 → ?\n".into())])].into()),
        ]),
    );
    let published = c.read();
    let problems = published.get("params").get("diagnostics").items();
    assert_eq!(problems.len(), 1);
    assert_eq!(problems[0].get("severity").num(), Some(1));
    assert_eq!(problems[0].get("range").get("start").get("line").num(), Some(1));

    let unknown = c.request("textDocument/typeDefinition", Client::at(0, 0));
    assert_eq!(
        unknown.get("error").get("message").str(),
        Some("method not found: textDocument/typeDefinition")
    );

    assert_eq!(c.request("shutdown", Json::Null).get("result"), &Json::Null);
    c.notify("exit", Json::Null);
    assert!(c.child.wait().unwrap().success());
}
