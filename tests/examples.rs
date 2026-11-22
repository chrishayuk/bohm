//! Runs every program in `examples/` and compares what it prints with the
//! `.out` file beside it. A `.in` file, if present, is the input tape.
//!
//! Each program is run twice: compiled by Böhm's compiler and executed on the
//! machine, and with its formulas read as algebra. A program whose first line
//! is `# machine only` is run on the machine alone.

use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

fn run(program: &Path, via: &str, input: &[u8]) -> Result<String, String> {
    let mut child = Command::new(env!("CARGO_BIN_EXE_bohm"))
        .arg("run")
        .arg(program)
        .args(["--via", via])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(input).unwrap();
    let output = child.wait_with_output().unwrap();
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

#[test]
fn examples_print_their_expected_output() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples");
    let mut programs: Vec<_> = fs::read_dir(&dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "bohm"))
        .collect();
    programs.sort();

    let mut failures = Vec::new();
    for program in &programs {
        let name = program.file_stem().unwrap().to_string_lossy().into_owned();
        let expected = fs::read_to_string(program.with_extension("out"))
            .unwrap_or_else(|_| panic!("{name}.bohm has no {name}.out"));
        let input = fs::read(program.with_extension("in")).unwrap_or_default();
        let machine_only = fs::read_to_string(program).unwrap().starts_with("# machine only");
        let ways: &[&str] = if machine_only { &["bohm"] } else { &["bohm", "interp"] };
        for via in ways {
            match run(program, via, &input) {
                Err(e) => failures.push(format!("{name} (--via {via}): {e}")),
                Ok(actual) if actual != expected => {
                    failures.push(format!("{name} (--via {via}): output differs from {name}.out"))
                }
                Ok(_) => {}
            }
        }
    }

    assert!(failures.is_empty(), "{}", failures.join("\n"));
    assert!(programs.len() >= 42, "only {} examples were found", programs.len());
}
