//! Corrado Böhm's 1951 language: machine, loader, compiler, and the tools to
//! check each against the others.

pub mod compile;
pub mod interp;
pub mod machine;
pub mod syntax;

use compile::{Overrides, Unit};
use machine::{Fault, Machine, PI};

pub const COMPILER_1951: &str = include_str!("../programs/compiler-1951.bohm");
pub const LOADER: &str = include_str!("../programs/loader.bohm");

pub const STEP_LIMIT: u64 = 50_000_000;

/// The cell of the letter m, the compiler's base constant.
const CELL_M: usize = 14;

/// A correction to chapter 7 as printed.
pub struct Erratum {
    pub id: &'static str,
    pub title: &'static str,
    pub detail: &'static str,
    edits: &'static [(&'static str, &'static str)],
}

pub const ERRATA: &[Erratum] = &[
    Erratum {
        id: "branch",
        title: "Group C chooses D and E at once",
        detail: "The choice at the end of group C is [1∩(r∸1)]·D + (1∩r)·E + (1∸r)·F. For a \
                 variable, an operation or → (r = 2, 3, 4) both the first and the second \
                 coefficient are 1, so π receives D + E. The coefficient of E must be 1 only \
                 for an opening parenthesis: 1 ∸ (r÷1), as group A writes it for B.",
        edits: &[("(1∩r)·E + (1∸r)·F → π", "[1 ∸ (r÷1)]·E + (1∸r)·F → π")],
    },
    Erratum {
        id: "store",
        title: "Group E′ leaves out the operation code of a store",
        detail: "As printed, `a → b` is encoded as h·m3 + k, a number with operation code 00, \
                 which the machine does not define. Section 4.4 encodes the same formula as \
                 5·m2 + C·m3 + F. One term is added.",
        edits: &[("\nh·m3 + k → ?\n", "\n5·m2 + h·m3 + k → ?\n")],
    },
    Erratum {
        id: "link",
        title: "Group I overwrites the link to the enclosing parenthesis",
        detail: "When a ( follows an operation, group I files the finished instruction number in \
                 the cell of the current parenthesis, which still holds F of the parenthesis \
                 around it. Knuth and Trabb Pardo (1976) report this bug without printing their \
                 fix; this one is ours. The instruction number is filed in the cell of the new \
                 parenthesis instead, where its last three digits still serve as the link, and \
                 group G moves it home when the outer parenthesis closes.",
        edits: &[
            ("\nm3·X + Y + n → ↓Y\n", "\nm3·X + Y + n → ↓X\n"),
            ("\nπ' → G\n", "\nπ' → G\nY → ↓Y\nY mod m3 → Y\n"),
        ],
    },
    Erratum {
        id: "names",
        title: "The compiler's variable X is also the X of the code it generates",
        detail: "Chapter 6 keeps the running product of a polynomial in the letter X (and the sum \
                 in S). Chapter 7 uses X for F(n) and is itself written with polynomials, so the \
                 compiler compiled by its own method destroys its X. Its X is renamed Z.",
        edits: &[],
    },
];

/// Chapter 7 with the chosen errata applied. `ids` may hold "all" or "none".
pub fn compiler_source(ids: &[&str]) -> Result<String, String> {
    let all = ids.contains(&"all");
    for id in ids {
        if !["all", "none"].contains(id) && !ERRATA.iter().any(|e| e.id == *id) {
            return Err(format!("unknown erratum `{}`", id));
        }
    }
    let mut text = COMPILER_1951.to_string();
    for e in ERRATA.iter().filter(|e| all || ids.contains(&e.id)) {
        for (old, new) in e.edits {
            assert_eq!(text.matches(old).count(), 1, "erratum {} must apply exactly once", e.id);
            text = text.replace(old, new);
        }
    }
    if all || ids.contains(&"names") {
        text = text
            .lines()
            .map(|line| match line.split_once('#') {
                _ if line.starts_with("const") => line.to_string(),
                Some((code, note)) => format!("{}#{}", code.replace('X', "Z"), note),
                None => line.replace('X', "Z"),
            })
            .collect::<Vec<_>>()
            .join("\n");
    }
    Ok(text)
}

/// The loader in absolute form, as the operator would key it in.
pub struct Loader {
    pub origin: usize,
    pub code: Vec<u64>,
    /// Cells the loader needs before it runs: its group addresses and constants.
    pub prefix: Vec<(usize, u64)>,
    pub alpha: u64,
    pub unit: Unit,
}

pub fn loader() -> Loader {
    let unit = Unit::build(LOADER, Overrides::default()).expect("loader source is valid");
    let origin = unit.layout.origin;
    let mut code = Vec::new();
    let mut prefix = Vec::new();
    for item in &unit.items {
        match item.label {
            Some(cell) => prefix.push((cell, (origin + code.len()) as u64)),
            None => item.normal.code(unit.m(), &mut code),
        }
    }
    prefix.extend(unit.layout.consts.iter().copied());
    let alpha = unit.layout.lookup("α").expect("loader has a group α");
    let alpha = prefix.iter().find(|p| p.0 == alpha).unwrap().1;
    Loader { origin, code, prefix, alpha, unit }
}

#[derive(Debug)]
pub struct Run {
    pub output: Vec<u64>,
    pub steps: u64,
    pub fault: Option<Fault>,
    pub input_left: usize,
}

/// Put the loader in memory, give it the tape, and let it load the program.
/// Returns the machine stopped, ready for the operator to start the program.
pub fn load(code: &[u64], prefix: &[(usize, u64)]) -> Result<Machine, String> {
    let l = loader();
    if let Some(last) = prefix.iter().find(|p| p.0 == machine::PI_PRIME) {
        let end = last.1 as usize + code.len();
        if end > l.origin {
            return Err(format!(
                "the program would run into the loader at {} (it needs up to {})",
                l.origin, end
            ));
        }
    }
    let mut m = Machine::new();
    m.mem[l.origin..l.origin + l.code.len()].copy_from_slice(&l.code);
    m.mem[PI] = l.origin as u64;
    for (a, v) in l.prefix.iter().chain(prefix) {
        m.input.extend([*a as u64, *v]);
    }
    m.input.extend([0, l.alpha]);
    m.input.extend(code);
    m.input.push_back(0);
    m.run(STEP_LIMIT).map_err(|f| format!("while loading: {}", f))?;
    if !m.input.is_empty() {
        return Err("the loader stopped before the end of the tape".into());
    }
    Ok(m)
}

/// Start a loaded program at the group whose cell is `start`.
pub fn start(m: Machine, start: usize, input: &[u64], limit: u64) -> (Run, Machine) {
    start_watched(m, start, input, limit, |_, _| {})
}

/// As `start`, calling `watch(address, instruction number)` before each step.
pub fn start_watched(
    mut m: Machine,
    start: usize,
    input: &[u64],
    limit: u64,
    mut watch: impl FnMut(usize, u64),
) -> (Run, Machine) {
    m.mem[PI] = m.mem[start];
    m.input = input.iter().copied().collect();
    m.output.clear();
    m.steps = 0;
    let fault = loop {
        let pc = m.mem[PI] as usize;
        if pc < machine::CELLS && pc != machine::IO {
            watch(pc, m.mem[pc]);
        }
        match m.step() {
            Ok(true) if m.steps >= limit => break Some(Fault::StepLimit),
            Ok(true) => {}
            Ok(false) => break None,
            Err(f) => break Some(f),
        }
    };
    let run =
        Run { output: m.output.clone(), steps: m.steps, fault, input_left: m.input.len() };
    (run, m)
}

/// Run a program under the algebraic reading of the notation.
pub fn run_interp(unit: &Unit, input: &[u64], limit: u64) -> Result<Run, String> {
    let mut i = interp::Interp::new(unit, input)?;
    let fault = i.run(limit).err();
    Ok(Run { output: i.output, steps: i.steps, fault, input_left: i.input.len() })
}

/// Load instruction numbers for `unit` and run them on the machine.
pub fn run_code(unit: &Unit, code: &[u64], input: &[u64], limit: u64) -> Result<Run, String> {
    let m = load(code, &unit.prefix())?;
    Ok(start(m, unit.start()?, input, limit).0)
}

/// How a compiler is being run.
pub enum Engine<'a> {
    /// Chapter 7 read as algebra.
    Interp,
    /// Chapter 7 as instruction numbers on the machine: the reference
    /// translation when `None`, otherwise the given numbers.
    Machine(Option<&'a [u64]>),
}

/// Run Böhm's compiler (`compiler`) on the teletypewriter tape `chars`,
/// with its base constant set to `m`. Returns the instruction numbers it wrote.
pub fn bohm_compile(
    compiler: &Unit,
    engine: Engine,
    chars: &[u64],
    m: u64,
) -> Result<Vec<u64>, String> {
    // The compiler keeps one cell per opening parenthesis of a formula, from m+1.
    let mut deepest = 0;
    let mut open = 0;
    for c in chars {
        match *c {
            compile::OPEN => open += 1,
            compile::ARROW => open = 0,
            _ => {}
        }
        deepest = deepest.max(open);
    }
    let (lo, hi) = (m as usize + 1, m as usize + deepest.max(1));
    for (what, a, b) in compiler.regions() {
        if lo <= b && a <= hi {
            return Err(format!(
                "m = {} puts the compiler's working cells on its own {} ({}..{})",
                m, what, a, b
            ));
        }
    }
    let run = match engine {
        Engine::Interp => {
            let mut i = interp::Interp::new(compiler, chars)?;
            i.mem[CELL_M] = m;
            let fault = i.run(STEP_LIMIT).err();
            Run { output: i.output, steps: i.steps, fault, input_left: i.input.len() }
        }
        Engine::Machine(code) => {
            let own;
            let code = match code {
                Some(c) => c,
                None => {
                    own = compiler.code();
                    &own
                }
            };
            let mut prefix = compiler.prefix();
            prefix.push((CELL_M, m));
            let machine = load(code, &prefix)?;
            start(machine, compiler.start()?, chars, STEP_LIMIT).0
        }
    };
    if let Some(f) = run.fault {
        return Err(format!(
            "the compiler failed after writing {} numbers, with {} symbols unread: {}",
            run.output.len(),
            run.input_left,
            f
        ));
    }
    if run.input_left > 0 {
        return Err(format!(
            "the compiler stopped after writing {} numbers, with {} symbols unread",
            run.output.len(),
            run.input_left
        ));
    }
    Ok(run.output)
}

/// Chapter 7, corrected, laid out with the given base for its own
/// intermediate results.
pub fn compiler(errata: &[&str], temps: Option<usize>) -> Result<Unit, String> {
    Unit::build(&compiler_source(errata)?, Overrides { temps, origin: None })
}
