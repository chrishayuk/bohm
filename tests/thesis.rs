//! Checks against the text of the thesis, against Knuth and Trabb Pardo, and
//! of the three readings of a program against each other.

// Instruction numbers are grouped as the thesis prints them: c1 cop c2 cr.
#![allow(clippy::inconsistent_digit_grouping)]

use bohm::compile::{Overrides, Unit};
use bohm::machine::{Fault, Machine};
use bohm::{bohm_compile, compiler, run_code, run_interp, Engine, ERRATA, STEP_LIMIT};

fn unit(text: &str) -> Unit {
    Unit::build(text, Overrides::default()).unwrap()
}

fn program(name: &str) -> String {
    std::fs::read_to_string(format!("{}/programs/{}", env!("CARGO_MANIFEST_DIR"), name)).unwrap()
}

fn listing(u: &Unit) -> Vec<String> {
    u.code().iter().map(|w| u.layout.disasm(*w)).collect()
}

fn by_bohm(u: &Unit) -> Result<Vec<u64>, String> {
    let c = compiler(&["all"], None).unwrap();
    bohm_compile(&c, Engine::Machine(None), &u.chars(), u.m())
}

/// All three readings of a program give `want`, and Böhm's compiler writes
/// the same instruction numbers as the reference translator.
fn three_ways(text: &str, input: &[u64], want: &[u64]) {
    let u = unit(text);
    let algebra = run_interp(&u, input, STEP_LIMIT).unwrap();
    assert_eq!(algebra.fault, None);
    assert_eq!(algebra.output, want, "formulas read as algebra");
    let reference = u.code();
    let machine = run_code(&u, &reference, input, STEP_LIMIT).unwrap();
    assert_eq!(machine.fault, None);
    assert_eq!(machine.output, want, "reference translation on the machine");
    assert_eq!(by_bohm(&u).unwrap(), reference, "Böhm's compiler against the reference");
}

/// Section 1.4, the six instruction numbers (1.42) equivalent to a
/// conditional jump: to 301 if the number at address 002 is positive, else 302.
#[test]
fn conditional_jump_of_section_1_4() {
    for (a, target) in [(5u64, 301u64), (0, 302)] {
        let mut m = Machine::new();
        let code: [u64; 6] = [
            7_09_0002_0028,
            7_03_0002_0029,
            28_02_0025_0030,
            29_02_0026_0031,
            30_01_0031_0300,
            5_0300_0000,
        ];
        m.mem[100..106].copy_from_slice(&code);
        m.mem[7] = 1;
        m.mem[25] = 301;
        m.mem[26] = 302;
        m.mem[2] = a;
        m.mem[0] = 100;
        m.run(100).unwrap();
        assert_eq!(m.mem[300], target);
        // The machine stopped on the empty cell it jumped to.
        assert_eq!(m.mem[0], target + 1);
    }
}

/// Section 2: the same six instructions in the new notation (2.7).
#[test]
fn notation_of_section_2_7() {
    let u = unit("1∩a → A\n 1∸a → B\n A·x → C\n B·y → D\n C+D → u\n u → π");
    // Written without brackets these are polynomials, three or more
    // instruction numbers each. Bracketed, each is one operation and a store.
    let v = unit("(1∩a) → A\n (1∸a) → B\n (A·x) → C\n (B·y) → D\n (C+D) → u\n u → π");
    assert_eq!(listing(&v)[0], "‹1›∩a → [081]");
    assert_eq!(listing(&v)[1], "[081] → A");
    assert!(u.code().len() > v.code().len());
}

/// Section 4.1: the loader recognises `π′ → K` by adding 9490010000.
#[test]
fn loader_recognises_group_headers() {
    let u = unit("π' → K\n Ω → π");
    let header = u.code()[0];
    assert_eq!(header, 5_0999_0038);
    assert_eq!((header - header % 10_000 + 9_490_010_000) % 10_000_000_000, 0);
    // The printing has 94999010000, which would not recognise it.
    assert_ne!((header - header % 10_000 + 94_999_010_000) % 10_000_000_000, 0);
}

/// Section 5.3: formula (5.31) must give the sequence (5.32).
#[test]
fn formula_5_31_gives_sequence_5_32() {
    let u = unit("(((a+b)·(c:d))∸((f∪g)∩h)) → x");
    assert_eq!(
        listing(&u),
        [
            "f∪g → [086]",       // f op5 g → x6
            "[086]∩h → [085]",   // x6 op6 h → x5
            "c:d → [084]",       // c op3 d → x4
            "a+b → [083]",       // a op1 b → x3
            "[083]·[084] → [082]", // x3 op2 x4 → x2
            "[082]∸[085] → [081]", // x2 op4 x5 → x1
            "[081] → x",
        ]
    );
    assert_eq!(by_bohm(&u).unwrap(), u.code());
}

/// Knuth and Trabb Pardo, p. 41: what Böhm's algorithm punches for
/// ((a:(b·c))+((d∩e)∸f)) → g once the bug in group I is removed.
#[test]
fn knuth_and_trabb_pardo_page_41() {
    let u = unit("π' → A\n ((a:(b·c))+((d∩e)∸f)) → g\n Ω → π");
    let code = by_bohm(&u).unwrap();
    let text: Vec<String> = code.iter().map(|w| u.layout.disasm(*w)).collect();
    assert_eq!(
        text[1..7],
        [
            "d∩e → [085]",
            "[085]∸f → [084]",
            "b·c → [083]",
            "a:[083] → [082]",
            "[082]+[084] → [081]",
            "[081] → g",
        ]
    );
}

/// Section 6.2: the four prototype polynomials of table (6.2). The table
/// leaves out the `0 → S` that group G′ writes first.
#[test]
fn polynomials_of_table_6_2() {
    let cases: [(&str, &[&str]); 4] = [
        ("a·b:c → d", &["Ω → S", "a → X", "X·b → X", "X:c → X", "S+X → S", "S → d"]),
        ("a∸b·c → d", &["a → S", "b → X", "X·c → X", "S∸X → S", "S → d"]),
        ("a+b∸c → d", &["a → S", "S+b → S", "S∸c → S", "S → d"]),
        ("a:b+c → d", &["Ω → S", "a → X", "X:b → X", "S+X → S", "S+c → S", "S → d"]),
    ];
    for (formula, want) in cases {
        let u = unit(formula);
        assert_eq!(listing(&u), want, "{}", formula);
        assert_eq!(by_bohm(&u).unwrap(), u.code(), "{}", formula);
    }
}

#[test]
fn euclid_for_two_numbers() {
    three_ways(&program("euclid.bohm"), &[1071, 462], &[21]);
    three_ways(&program("euclid.bohm"), &[17, 5], &[1]);
}

/// Group E ends without a jump, as printed on p. 26, and falls through into
/// the group after it, whose header the printing gives as a second E.
#[test]
fn euclid_for_n_numbers_group_e_as_printed() {
    let text = program("euclid-n.bohm");
    let lines: Vec<&str> = text
        .lines()
        .map(|l| l.split('#').next().unwrap().trim())
        .filter(|l| !l.is_empty())
        .collect();
    let e = lines.iter().position(|l| *l == "π' → E").unwrap();
    assert_eq!(lines[e..e + 4], ["π' → E", "100 → i", "101 → k", "π' → F"]);
}

#[test]
fn euclid_for_n_numbers() {
    three_ways(&program("euclid-n.bohm"), &[4, 1071, 462, 84, 147], &[21]);
    three_ways(&program("euclid-n.bohm"), &[3, 30, 42, 70], &[2]);
}

const TPK_IN: [u64; 11] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
const TPK_OUT: [u64; 22] =
    [10, 999, 9, 999, 8, 999, 7, 999, 6, 999, 5, 999, 4, 322, 3, 136, 2, 41, 1, 6, 0, 0];

/// Knuth and Trabb Pardo's TPK program is right as algebra, but it names a
/// group S and a variable X, the two letters chapter 6 computes polynomials in.
#[test]
fn tpk_as_printed_does_not_survive_compilation() {
    let u = unit(&program("tpk.bohm"));
    let algebra = run_interp(&u, &TPK_IN, STEP_LIMIT).unwrap();
    assert_eq!(algebra.output, TPK_OUT);
    let code = by_bohm(&u).unwrap();
    assert_eq!(code, u.code());
    let machine = run_code(&u, &code, &TPK_IN, STEP_LIMIT).unwrap();
    assert_eq!(machine.output, []);
}

#[test]
fn tpk_with_s_and_x_renamed() {
    three_ways(&program("tpk-renamed.bohm"), &TPK_IN, &TPK_OUT);
    let rename = |text: &str| -> Vec<String> {
        text.lines()
            .map(|l| l.split('#').next().unwrap().trim().to_string())
            .filter(|l| !l.is_empty())
            .collect()
    };
    let printed: Vec<String> =
        rename(&program("tpk.bohm")).iter().map(|l| l.replace('S', "K").replace('X', "Z")).collect();
    assert_eq!(printed, rename(&program("tpk-renamed.bohm")));
}

fn selfhost(errata: &[&str]) -> Result<(), String> {
    let (a, b) = (compiler(errata, Some(800))?, compiler(errata, Some(850))?);
    let chars = a.chars();
    let s0 = bohm_compile(&a, Engine::Interp, &chars, 850)?;
    if s0 != b.code() {
        return Err("stage 0 differs from the reference".into());
    }
    let s1 = bohm_compile(&b, Engine::Machine(Some(&s0)), &chars, 800)?;
    if s1 != a.code() {
        return Err("stage 1 differs from the reference".into());
    }
    let s2 = bohm_compile(&a, Engine::Machine(Some(&s1)), &chars, 850)?;
    if s2 != s0 {
        return Err("stage 2 differs from stage 0".into());
    }
    Ok(())
}

/// The compiler, compiled by itself and run on the machine, writes itself.
#[test]
fn the_compiler_compiles_itself() {
    selfhost(&["all"]).unwrap();
}

/// Chapter 7 as printed does not get through its own text, and neither does
/// it with any one of the corrections left out.
#[test]
fn every_erratum_is_needed() {
    assert!(selfhost(&["none"]).is_err());
    for skip in ERRATA {
        let rest: Vec<&str> = ERRATA.iter().map(|e| e.id).filter(|id| *id != skip.id).collect();
        assert!(selfhost(&rest).is_err(), "self-compilation succeeded without `{}`", skip.id);
    }
}

/// Without the renaming, chapter 7 still works when read as algebra: the
/// clash only exists once the compiler is itself instruction numbers.
#[test]
fn the_name_clash_only_shows_on_the_machine() {
    let errata = ["branch", "store", "link"];
    let c = compiler(&errata, None).unwrap();
    let u = unit(&program("euclid-n.bohm"));
    assert_eq!(bohm_compile(&c, Engine::Interp, &u.chars(), u.m()).unwrap(), u.code());
    assert!(bohm_compile(&c, Engine::Machine(None), &u.chars(), u.m()).is_err());
}

/// A formula that reads `?` more than once reads it in the order of the
/// instructions written for it, in every reading: a polynomial left to right,
/// a bracketed formula in the order of (5.32), inner brackets first. The
/// review of 22 November 2026 found the algebraic reading printing 7 here
/// while the machine printed 5.
#[test]
fn input_is_read_in_the_order_of_the_instructions() {
    three_ways("π' → A\n?+?·? → ?\nΩ → π", &[1, 2, 3], &[7]);
    // The multiplication is region 2 and is computed first: 1·2, then 3 + 2.
    three_ways("π' → A\n(?+(?·?)) → ?\nΩ → π", &[1, 2, 3], &[5]);
    // The right-hand bracket is region 3 and is computed first: 1∸2, 10∸4, 6∸0.
    three_ways("π' → A\n((?∸?)∸(?∸?)) → ?\nΩ → π", &[1, 2, 10, 4], &[6]);
}

/// Formulas as printed in chapter 7, each with its operations put in
/// brackets: a formula with an operation in it becomes `(…) → …`.
fn bracket_every_formula(text: &str) -> String {
    text.lines()
        .map(|line| {
            let code = line.split('#').next().unwrap().trim();
            match code.split_once('→') {
                Some((lhs, rhs))
                    if lhs.chars().any(|c| "+∸·:÷∪∩".contains(c)) || lhs.contains("mod") =>
                {
                    format!("({}) → {}", lhs.trim(), rhs.trim())
                }
                _ => line.to_string(),
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// The fourth correction, `names`, is needed only because the compiler's
/// formulas without brackets are compiled as chapter 6 polynomials, which
/// work in S and X. With every formula bracketed instead, the compiler keeps
/// its X and still compiles itself with the other three corrections.
#[test]
fn names_is_needed_only_for_polynomials() {
    let text = bracket_every_formula(&bohm::compiler_source(&["branch", "store", "link"]).unwrap());
    let build = |m| Unit::build(&text, Overrides { temps: Some(m), origin: None }).unwrap();
    let (a, b) = (build(800), build(850));
    let chars = a.chars();
    let s0 = bohm_compile(&a, Engine::Interp, &chars, 850).unwrap();
    assert_eq!(s0, b.code());
    let s1 = bohm_compile(&b, Engine::Machine(Some(&s0)), &chars, 800).unwrap();
    assert_eq!(s1, a.code());
    let s2 = bohm_compile(&a, Engine::Machine(Some(&s1)), &chars, 850).unwrap();
    assert_eq!(s2, s0);
}

struct Rng(u64);

impl Rng {
    fn next(&mut self, n: u64) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (self.0 >> 33) % n
    }
    fn pick<'a>(&mut self, xs: &[&'a str]) -> &'a str {
        xs[self.next(xs.len() as u64) as usize]
    }
}

const OPS: [&str; 8] = ["+", "∸", "·", ":", "÷", " mod ", "∪", "∩"];

fn bracketed(r: &mut Rng, depth: u32, atoms: &[&str]) -> String {
    if depth == 0 || r.next(10) < 3 {
        return r.pick(atoms).to_string();
    }
    let l = bracketed(r, depth - 1, atoms);
    let op = r.pick(&OPS);
    format!("({}{}{})", l, op, bracketed(r, depth - 1, atoms))
}

fn polynomial(r: &mut Rng, atoms: &[&str]) -> String {
    let mut s = String::new();
    for i in 0..1 + r.next(4) {
        if i > 0 {
            s += r.pick(&["+", "∸"]);
        }
        for j in 0..1 + r.next(3) {
            if j > 0 {
                s += r.pick(&["·", ":"]);
            }
            s += r.pick(atoms);
        }
    }
    s
}

fn formula(r: &mut Rng, atoms: &[&str]) -> String {
    if r.next(2) == 0 {
        let l = bracketed(r, 3, atoms);
        let op = r.pick(&OPS);
        format!("({}{}{})", l, op, bracketed(r, 3, atoms))
    } else {
        polynomial(r, atoms)
    }
}

/// Böhm's compiler and the reference translator agree, number for number,
/// on random formulas of both accepted shapes.
#[test]
fn random_formulas_translate_alike() {
    let atoms = ["a", "b", "c", "d", "↓e", "↓f", "π", "?", "Ω", "7", "12345678901234", "Q", "g1"];
    let dests = ["a", "b", "↓c", "π", "?", "Q", "g1"];
    let mut r = Rng(1951);
    for round in 0..20 {
        let mut text = String::from("temps 100\nπ' → A\n");
        for _ in 0..40 {
            text += &format!("{} → {}\n", formula(&mut r, &atoms), r.pick(&dests));
        }
        let u = unit(&text);
        assert_eq!(by_bohm(&u).unwrap(), u.code(), "round {}:\n{}", round, text);
    }
}

/// A formula means the same read as algebra and run as the instruction
/// numbers Böhm's compiler writes for it, overflow, division by zero and
/// formulas that read `?` more than once included.
#[test]
fn random_formulas_mean_the_same() {
    let atoms = ["a", "b", "c", "d", "e", "f", "2", "3", "100000000", "?"];
    let mut r = Rng(1954);
    let mut faults = 0;
    for round in 0..150 {
        let text = format!(
            "temps 100\nπ' → A\n? → a\n? → b\n? → c\n? → d\n? → e\n? → f\n{} → z\nz → ?\nΩ → π\n",
            formula(&mut r, &atoms)
        );
        let input: Vec<u64> = (0..40)
            .map(|_| if r.next(4) == 0 { 99_999_999_999_000 + r.next(1000) } else { r.next(12) })
            .collect();
        let u = unit(&text);
        let algebra = run_interp(&u, &input, STEP_LIMIT).unwrap();
        let machine = run_code(&u, &by_bohm(&u).unwrap(), &input, STEP_LIMIT).unwrap();
        assert_eq!(
            (&algebra.output, &algebra.fault),
            (&machine.output, &machine.fault),
            "round {} with input {:?}:\n{}",
            round,
            input,
            text
        );
        if algebra.fault == Some(Fault::DivideByZero) {
            faults += 1;
        }
    }
    assert!(faults > 0 && faults < 150, "the sample should mix clean runs and faults");
}
