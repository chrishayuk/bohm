use bohm::compile::{Overrides, Unit};
use bohm::machine::show_word;
use bohm::{bohm_compile, compiler, run_code, run_interp, Engine, Run, ERRATA, STEP_LIMIT};
use std::io::{IsTerminal, Read};
use std::process::ExitCode;

const USAGE: &str = "\
bohm: Corrado Böhm's 1951 language, machine, loader and compiler

usage:
  bohm run FILE [--input N,N,...] [--via interp|ref|bohm] [--trace] [--errata LIST]
        Run a program. `interp` reads the formulas as algebra; `ref` compiles
        with the reference translator and runs on the machine; `bohm` compiles
        with Böhm's own compiler running on the machine (the default).
        --trace prints every instruction the machine executes.
        Without --input, the numbers are read from standard input.
        FILE may be `-`: standard input is then the program, a line `----`,
        and the numbers.
  bohm compile FILE [--emit listing|normal|chars|code] [--via ref|bohm] [--errata LIST]
        Show a program as accepted formulas, teletypewriter numbers or
        instruction numbers.
  bohm verify FILE [--input N,N,...] [--errata LIST]
        Run all three ways and compare.
  bohm selfhost [--errata LIST]
        Have Böhm's compiler compile itself.
  bohm errata
        List the corrections to chapter 7.
  bohm check FILE
        List what is wrong with a program, and what compiles but is probably
        not what was meant, one per line as FILE:LINE:COLUMN: KIND: MESSAGE.
  bohm format FILE
        Print the program with every symbol written as the thesis prints it.
  bohm lsp
        Serve editors, speaking the Language Server Protocol on standard
        input and output.

  --errata takes `all` (the default), `none`, or names from `bohm errata`.
  --compiler machine|algebra chooses how Böhm's compiler itself is run: as
        instruction numbers on the machine (the default), or with its formulas
        read as algebra.";

struct Args {
    rest: Vec<String>,
}

impl Args {
    fn opt(&mut self, name: &str) -> Option<String> {
        let i = self.rest.iter().position(|a| a == name)?;
        if i + 1 >= self.rest.len() {
            return None;
        }
        self.rest.remove(i);
        Some(self.rest.remove(i))
    }
}

fn numbers(s: Option<String>) -> Result<Vec<u64>, String> {
    match s {
        None => Ok(Vec::new()),
        Some(s) => s
            .split(|c: char| c == ',' || c.is_whitespace())
            .filter(|p| !p.is_empty())
            .map(|p| p.parse::<u64>().map_err(|_| format!("`{}` is not a number", p)))
            .collect(),
    }
}

fn report(run: &Run) -> Result<(), String> {
    for v in &run.output {
        println!("{}", v);
    }
    match &run.fault {
        Some(f) => Err(format!("stopped: {}", f)),
        None => Ok(()),
    }
}

fn via_bohm(unit: &Unit, errata: &[&str], algebra: bool) -> Result<Vec<u64>, String> {
    let c = compiler(errata, None)?;
    let engine = if algebra { Engine::Interp } else { Engine::Machine(None) };
    bohm_compile(&c, engine, &unit.chars(), unit.m())
}

fn main() -> ExitCode {
    match real_main() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("bohm: {}", e);
            ExitCode::FAILURE
        }
    }
}

fn real_main() -> Result<(), String> {
    let mut args = Args { rest: std::env::args().skip(1).collect() };
    if args.rest.first().map(String::as_str) == Some("lsp") {
        let stdin = std::io::stdin();
        return bohm::lsp::serve(&mut stdin.lock(), &mut std::io::stdout());
    }
    let errata_arg = args.opt("--errata").unwrap_or("all".into());
    let errata: Vec<&str> = errata_arg.split(',').collect();
    // FILE may be `-`: standard input is then the program, a line of four
    // dashes, and the numbers the program reads. The playground uses this.
    let piped = if args.rest.get(1).map(String::as_str) == Some("-") {
        let mut text = String::new();
        std::io::stdin().read_to_string(&mut text).map_err(|e| e.to_string())?;
        let (program, numbers) = match text.split_once("\n----\n") {
            Some((program, numbers)) => (program.to_string(), numbers.to_string()),
            None => (text, String::new()),
        };
        Some((program, numbers))
    } else {
        None
    };
    let input = match (args.opt("--input"), &piped) {
        (Some(list), _) => numbers(Some(list))?,
        (None, Some((_, text))) => numbers(Some(text.clone()))?,
        // With no --input, numbers come from standard input when it is a file or a pipe.
        (None, None) if !std::io::stdin().is_terminal() && !["check", "format"].contains(&args.rest.first().map(String::as_str).unwrap_or("")) => {
            let mut text = String::new();
            std::io::stdin().read_to_string(&mut text).map_err(|e| e.to_string())?;
            numbers(Some(text))?
        }
        (None, None) => Vec::new(),
    };
    let via = args.opt("--via");
    let emit = args.opt("--emit").unwrap_or("listing".into());
    let algebra = match args.opt("--compiler").as_deref() {
        None | Some("machine") => false,
        Some("algebra") => true,
        Some(v) => return Err(format!("unknown --compiler {}", v)),
    };
    let trace = match args.rest.iter().position(|a| a == "--trace") {
        Some(i) => {
            args.rest.remove(i);
            true
        }
        None => false,
    };
    let cmd = args.rest.first().cloned().unwrap_or_default();
    let file = || -> Result<Unit, String> {
        if let Some((program, _)) = &piped {
            return Unit::build(program, Overrides::default());
        }
        let path = args.rest.get(1).ok_or("which file?")?;
        let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {}", path, e))?;
        Unit::build(&text, Overrides::default()).map_err(|e| format!("{}: {}", path, e))
    };
    match cmd.as_str() {
        "run" => {
            let unit = file()?;
            let code = match via.as_deref().unwrap_or("bohm") {
                "interp" => return report(&run_interp(&unit, &input, STEP_LIMIT)?),
                "ref" => unit.code(),
                "bohm" => via_bohm(&unit, &errata, algebra)?,
                v => return Err(format!("unknown --via {}", v)),
            };
            if !trace {
                return report(&run_code(&unit, &code, &input, STEP_LIMIT)?);
            }
            let machine = bohm::load(&code, &unit.prefix())?;
            let (run, _) = bohm::start_watched(machine, unit.start()?, &input, STEP_LIMIT, |at, word| {
                eprintln!("{:03}  {}   {}", at, show_word(word), unit.layout.disasm(word));
            });
            report(&run)
        }
        "compile" => {
            let unit = file()?;
            let code = match via.as_deref().unwrap_or("ref") {
                "ref" => unit.code(),
                "bohm" => via_bohm(&unit, &errata, algebra)?,
                v => return Err(format!("unknown --via {}", v)),
            };
            match emit.as_str() {
                "normal" => {
                    for item in &unit.items {
                        println!("{}", item.normal.show(&unit.layout));
                    }
                }
                "chars" => {
                    for c in unit.chars() {
                        println!("{}", c);
                    }
                }
                "code" => {
                    for w in code {
                        println!("{:014}", w);
                    }
                }
                "listing" => {
                    for (what, a, b) in unit.regions() {
                        println!("# {:<22} {:03}..{:03}", what, a, b);
                    }
                    let mut at = unit.layout.origin;
                    for w in code {
                        let text = unit.layout.disasm(w);
                        if text.starts_with("π′ → ") {
                            println!("     {}   {}", show_word(w), text);
                        } else {
                            println!("{:03}  {}   {}", at, show_word(w), text);
                            at += 1;
                        }
                    }
                }
                e => return Err(format!("unknown --emit {}", e)),
            }
            Ok(())
        }
        "verify" => {
            let unit = file()?;
            verify(&unit, &input, &errata, algebra)
        }
        "selfhost" => selfhost(&errata),
        "check" | "format" => {
            let (name, text) = match &piped {
                Some((program, _)) => ("-".to_string(), program.clone()),
                None => {
                    let path = args.rest.get(1).ok_or("which file?")?;
                    (path.clone(), std::fs::read_to_string(path).map_err(|e| format!("{}: {}", path, e))?)
                }
            };
            if cmd == "format" {
                print!("{}", bohm::check::symbolise(&text));
                return Ok(());
            }
            let problems = bohm::check::check(&text);
            for d in &problems {
                let kind = if d.error { "error" } else { "warning" };
                println!("{}:{}:{}: {}: {}", name, d.line, d.start + 1, kind, d.message);
            }
            match problems.iter().filter(|d| d.error).count() {
                0 => Ok(()),
                n => Err(format!("{} error{}", n, if n == 1 { "" } else { "s" })),
            }
        }
        "errata" => {
            for e in ERRATA {
                println!("{}\n    {}\n    {}\n", e.id, e.title, e.detail);
            }
            Ok(())
        }
        _ => {
            println!("{}", USAGE);
            Ok(())
        }
    }
}

fn outcome(run: &Result<Run, String>) -> String {
    match run {
        Err(e) => format!("could not run: {}", e),
        Ok(r) => {
            let out: Vec<String> = r.output.iter().map(|v| v.to_string()).collect();
            match &r.fault {
                None => format!("output [{}] in {} steps", out.join(", "), r.steps),
                Some(f) => format!("output [{}] then {} after {} steps", out.join(", "), f, r.steps),
            }
        }
    }
}

fn verify(unit: &Unit, input: &[u64], errata: &[&str], as_algebra: bool) -> Result<(), String> {
    let reference = unit.code();
    let algebra = run_interp(unit, input, STEP_LIMIT);
    let machine = run_code(unit, &reference, input, STEP_LIMIT);
    println!("formulas read as algebra:        {}", outcome(&algebra));
    println!("reference translation, machine:  {}", outcome(&machine));
    let mut ok = match (&algebra, &machine) {
        (Ok(a), Ok(b)) => a.fault.is_none() && b.fault.is_none() && a.output == b.output,
        _ => false,
    };
    match via_bohm(unit, errata, as_algebra) {
        Err(e) => {
            println!("Böhm's compiler, machine:        {}", e);
            ok = false;
        }
        Ok(code) => {
            let run = run_code(unit, &code, input, STEP_LIMIT);
            println!("Böhm's compiler, machine:        {}", outcome(&run));
            let same = code == reference;
            println!(
                "instruction numbers:             Böhm's compiler wrote {}, the reference {}: {}",
                code.len(),
                reference.len(),
                if same { "identical" } else { "DIFFERENT" }
            );
            ok = ok && same;
        }
    }
    if ok {
        println!("all three agree");
        Ok(())
    } else {
        Err("the three readings do not agree".into())
    }
}

fn selfhost(errata: &[&str]) -> Result<(), String> {
    // The compiler keeps its parenthesis stack in the cells above m, and the
    // code it writes keeps intermediate results there too. A compiler cannot
    // therefore be compiled with its own m: two bases alternate.
    let (m_a, m_b) = (800, 850);
    let a = compiler(errata, Some(m_a))?;
    let b = compiler(errata, Some(m_b))?;
    let chars = a.chars();
    let (ref_a, ref_b) = (a.code(), b.code());
    println!(
        "chapter 7: {} formulas, {} teletypewriter symbols, {} instruction numbers",
        a.items.len(),
        chars.len(),
        ref_a.len()
    );
    let stage = |n: usize, by: &str, code: Vec<u64>, want: &Vec<u64>| -> Result<Vec<u64>, String> {
        let same = &code == want;
        println!(
            "stage {}: {} wrote {} numbers: {}",
            n,
            by,
            code.len(),
            if same { "identical to the reference translation" } else { "DIFFERENT" }
        );
        if same {
            Ok(code)
        } else {
            let at = code.iter().zip(want).position(|(x, y)| x != y).unwrap_or(code.len().min(want.len()));
            Err(format!("stage {} differs from the reference at number {}", n, at))
        }
    };
    let s0 = bohm_compile(&a, Engine::Interp, &chars, m_b as u64)?;
    let s0 = stage(0, "chapter 7 read as algebra", s0, &ref_b)?;
    let s1 = bohm_compile(&b, Engine::Machine(Some(&s0)), &chars, m_a as u64)?;
    let s1 = stage(1, "the stage 0 numbers, on the machine,", s1, &ref_a)?;
    let s2 = bohm_compile(&a, Engine::Machine(Some(&s1)), &chars, m_b as u64)?;
    let s2 = stage(2, "the stage 1 numbers, on the machine,", s2, &ref_b)?;
    if s2 == s0 {
        println!("stage 2 equals stage 0: the compiler reproduces itself");
        Ok(())
    } else {
        Err("stage 2 differs from stage 0".into())
    }
}
