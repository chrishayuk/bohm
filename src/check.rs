//! What an editor needs to know about a program: problems, what a symbol
//! means, where a group begins, and what a formula compiles to.
//!
//! Columns count UTF-16 code units from 0, as the Language Server Protocol
//! does. Lines count from 1, as the parser's messages do.

use crate::compile::{Normal, Overrides, Unit};
use crate::machine::{show_word, Op};
use crate::syntax::{Cell, Expr, Formula};
use std::collections::{BTreeMap, HashSet};

#[derive(Debug, Clone, PartialEq)]
pub struct Diag {
    pub line: usize,
    pub start: usize,
    pub end: usize,
    pub error: bool,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Kind {
    Name(String),
    Num(u64),
    Op(Op),
    Arrow,
    Down,
    Open,
    Close,
    /// `const`, `origin`, `temps` or `cells` at the start of a line.
    Directive,
    Other,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: Kind,
    pub start: usize,
    pub end: usize,
    /// The text as written.
    pub text: String,
}

const DIRECTIVES: [&str; 4] = ["const", "origin", "temps", "cells"];

/// The tokens of one line, without its comment. This follows the lexer in
/// `syntax.rs` but never fails: what it cannot read is `Kind::Other`.
pub fn tokens(line: &str) -> Vec<Token> {
    let code = line.split('#').next().unwrap();
    let cs: Vec<char> = code.chars().collect();
    let col = |i: usize| cs[..i].iter().map(|c| c.len_utf16()).sum::<usize>();
    let mut out = Vec::new();
    let mut i = 0;
    let suffix = |i: &mut usize, name: &mut String| {
        while *i < cs.len() && cs[*i].is_ascii_digit() {
            name.push(cs[*i]);
            *i += 1;
        }
        while *i < cs.len() && (cs[*i] == '\'' || cs[*i] == '′') {
            name.push('′');
            *i += 1;
        }
    };
    while i < cs.len() {
        let c = cs[i];
        let from = i;
        if c.is_whitespace() {
            i += 1;
            continue;
        }
        let kind = if c.is_ascii_digit() {
            let mut v: u64 = 0;
            while i < cs.len() && cs[i].is_ascii_digit() {
                v = v.saturating_mul(10).saturating_add(cs[i] as u64 - '0' as u64);
                i += 1;
            }
            Kind::Num(v)
        } else if c.is_ascii_alphabetic() {
            let mut word = String::new();
            while i < cs.len() && cs[i].is_ascii_alphabetic() {
                word.push(cs[i]);
                i += 1;
            }
            match word.as_str() {
                "mod" => Kind::Op(Op::Mod),
                "max" => Kind::Op(Op::Max),
                "min" => Kind::Op(Op::Min),
                "pi" => {
                    let mut n = "π".to_string();
                    suffix(&mut i, &mut n);
                    Kind::Name(n)
                }
                "Omega" => Kind::Name("Ω".into()),
                w if out.is_empty() && DIRECTIVES.contains(&w) => Kind::Directive,
                w if w.len() == 1 => {
                    let mut n = w.to_string();
                    suffix(&mut i, &mut n);
                    Kind::Name(n)
                }
                _ => Kind::Other,
            }
        } else {
            i += 1;
            match c {
                '→' => Kind::Arrow,
                '-' if i < cs.len() && cs[i] == '>' => {
                    i += 1;
                    Kind::Arrow
                }
                '↓' | '@' => Kind::Down,
                '(' | '[' | '{' => Kind::Open,
                ')' | ']' | '}' => Kind::Close,
                '+' => Kind::Op(Op::Add),
                '∸' | '-' | '−' => Kind::Op(Op::Sub),
                '·' | '*' => Kind::Op(Op::Mul),
                ':' | '/' => Kind::Op(Op::Div),
                '÷' | '~' => Kind::Op(Op::Diff),
                '∪' => Kind::Op(Op::Max),
                '∩' => Kind::Op(Op::Min),
                '?' => Kind::Name("?".into()),
                c if c.is_alphabetic() => {
                    let mut n = c.to_string();
                    suffix(&mut i, &mut n);
                    Kind::Name(n)
                }
                _ => Kind::Other,
            }
        };
        out.push(Token { kind, start: col(from), end: col(i), text: cs[from..i].iter().collect() });
    }
    out
}

fn is_directive(line: &str) -> bool {
    matches!(tokens(line).first(), Some(Token { kind: Kind::Directive, .. }))
}

/// The extent of a line's code, for a problem that belongs to the whole line.
fn extent(line: &str) -> (usize, usize) {
    let t = tokens(line);
    match (t.first(), t.last()) {
        (Some(a), Some(b)) => (a.start, b.end),
        _ => (0, line.encode_utf16().count().max(1)),
    }
}

fn names(e: &Expr, out: &mut Vec<String>) {
    match e {
        Expr::Cell(c) => out.push(c.name.clone()),
        Expr::Num(_) => {}
        Expr::Bin(l, _, r) => {
            names(l, out);
            names(r, out);
        }
        Expr::Paren(inner) => names(inner, out),
    }
}

fn label(f: &Formula) -> Option<&str> {
    match &f.src {
        Expr::Cell(Cell { name, ind: false }) if name == "π′" && !f.dst.ind => Some(&f.dst.name),
        _ => None,
    }
}

fn special(name: &str) -> bool {
    matches!(name, "π" | "π′" | "Ω" | "?")
}

/// Every problem in a program: what stops it compiling, and what compiles
/// but is probably not what was meant.
pub fn check(text: &str) -> Vec<Diag> {
    let lines: Vec<&str> = text.lines().collect();
    let whole = |n: usize, error: bool, message: String| {
        let (start, end) = extent(lines.get(n.saturating_sub(1)).copied().unwrap_or(""));
        Diag { line: n.max(1), start, end, error, message }
    };
    let unit = match Unit::build(text, Overrides::default()) {
        Ok(unit) => unit,
        Err(e) => {
            let at = e
                .strip_prefix("line ")
                .and_then(|rest| rest.split_once(": "))
                .and_then(|(n, msg)| Some((n.parse::<usize>().ok()?, msg.to_string())));
            return vec![match at {
                Some((n, msg)) => whole(n, true, msg),
                None => whole(1, true, e),
            }];
        }
    };
    let mut out = Vec::new();
    let formulas: Vec<&Formula> = unit.items.iter().map(|i| &i.formula).collect();
    if formulas.is_empty() {
        return out;
    }
    if let Err(e) = unit.start() {
        out.push(whole(formulas[0].line, true, e));
    }

    // Every name that is given a value somewhere.
    let mut set: HashSet<String> = HashSet::new();
    let mut groups: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for f in &formulas {
        if !f.dst.ind {
            set.insert(f.dst.name.clone());
        }
        if let Some(name) = label(f) {
            groups.entry(name.to_string()).or_default().push(f.line);
        }
    }
    for line in &lines {
        if let [Token { kind: Kind::Directive, text, .. }, Token { kind: Kind::Name(n), .. }, ..] =
            tokens(line).as_slice()
        {
            if text == "const" {
                set.insert(n.clone());
            }
        }
    }

    for (name, at) in &groups {
        for line in &at[1..] {
            out.push(whole(
                *line,
                false,
                format!("group {} already begins on line {}; the loader keeps the last one", name, at[0]),
            ));
        }
    }

    // S and X belong to the compiled form of every polynomial formula.
    let poly = unit
        .items
        .iter()
        .find(|i| matches!(i.normal, Normal::Poly { .. }))
        .map(|i| i.formula.line);
    // Names that a jump relies on.
    let mut targets: BTreeMap<usize, Vec<String>> = BTreeMap::new();
    for f in &formulas {
        if f.dst.name == "π" && !f.dst.ind {
            names(&f.src, targets.entry(f.line).or_default());
        }
    }
    let mut reported: HashSet<String> = HashSet::new();
    for (n, line) in lines.iter().enumerate() {
        if is_directive(line) {
            continue;
        }
        for t in tokens(line) {
            let Kind::Name(name) = &t.kind else { continue };
            if let (Some(p), "S" | "X") = (poly, name.as_str()) {
                let what = if name == "S" { "sum" } else { "product" };
                out.push(Diag {
                    line: n + 1,
                    start: t.start,
                    end: t.end,
                    error: false,
                    message: format!(
                        "{} is where compiled code keeps the running {} of a formula without \
                         brackets, such as line {}. On the machine they overwrite each other. \
                         Use another letter.",
                        name, what, p
                    ),
                });
            }
            let jump = targets.get(&(n + 1)).is_some_and(|names| names.contains(name));
            if jump && !special(name) && !set.contains(name) && reported.insert(name.clone()) {
                out.push(Diag {
                    line: n + 1,
                    start: t.start,
                    end: t.end,
                    error: false,
                    message: format!(
                        "{} is never given a value: no group begins with π' → {}, so this \
                         continues at 0 and stops",
                        name, name
                    ),
                });
            }
        }
    }
    out.sort_by_key(|d| (d.line, d.start));
    out
}

/// The address at which each formula's instructions begin, with the
/// instruction numbers themselves.
fn placed(unit: &Unit) -> Vec<(usize, Vec<u64>)> {
    let mut at = unit.layout.origin;
    unit.items
        .iter()
        .map(|item| {
            let mut words = Vec::new();
            item.normal.code(unit.m(), &mut words);
            let here = at;
            if item.label.is_none() {
                at += words.len();
            }
            (here, words)
        })
        .collect()
}

fn op_note(op: Op) -> (&'static str, &'static str) {
    match op {
        Op::Add => ("sum, keeping the last 14 digits", "+"),
        Op::Mul => ("product, keeping the last 14 digits", "*"),
        Op::Sub => ("difference, or 0 if it would be negative", "-"),
        Op::Div => ("quotient, rounded down", "/"),
        Op::Diff => ("absolute difference", "~"),
        Op::Mod => ("remainder", "mod"),
        Op::Max => ("the greater of the two", "max"),
        Op::Min => ("the smaller of the two", "min"),
    }
}

fn describe(t: &Token, unit: Option<&Unit>, groups: &BTreeMap<String, (usize, usize)>) -> String {
    let cell = |name: &str| unit.and_then(|u| u.layout.lookup(name));
    match &t.kind {
        Kind::Arrow => "`→` becomes: the value on the left is stored in the variable on the right. \
                        Also typed `->`."
            .into(),
        Kind::Down => "`↓x` is the cell whose address is in x. Also typed `@x`.".into(),
        Kind::Op(op) => {
            let (what, ascii) = op_note(*op);
            let sym = op.symbol().trim();
            let typed = if ascii == sym { String::new() } else { format!(" Also typed `{}`.", ascii) };
            format!("`{}` {}. Operation {:02}.{}", sym, what, op.code(), typed)
        }
        Kind::Open | Kind::Close => "Round, square and curly brackets are interchangeable.".into(),
        Kind::Directive => match t.text.as_str() {
            "const" => "`const NAME = N`: the cell NAME holds N when the program starts.".into(),
            "origin" => "`origin N`: load the program from cell N. The default is 500.".into(),
            "temps" => "`temps N`: intermediate results go to N+1, N+2, … The default is 80.".into(),
            _ => "`cells N`: numbers and extra names go to N, N+1, … The default is 54.".into(),
        },
        Kind::Num(0) => "`0` is Ω, cell 998, which holds 0.".into(),
        Kind::Num(v) => match cell(&v.to_string()) {
            Some(a) => format!("The number {}: cell {:03}, which holds {} from the start.", v, a, v),
            None => format!("The number {}.", v),
        },
        Kind::Name(n) => match n.as_str() {
            "?" => "`?` is cell 001, the outside world. On the left of → it is the next number \
                    of input; on the right it is the output."
                .into(),
            "π" => "`π` is cell 000, the address of the next instruction. Storing in it is a \
                    jump. Also typed `pi`."
                .into(),
            "π′" => "`π′` is cell 999, the address where the next formula will be. `π′ → K` \
                     names the group that begins here K. Also typed `pi'`."
                .into(),
            "Ω" => "`Ω` is cell 998, which holds 0. `Ω → π` stops. Also typed `Omega`.".into(),
            name => {
                let mut s = match (groups.get(name), cell(name)) {
                    (Some((line, at)), Some(a)) => format!(
                        "Group `{}`: cell {:03}, which holds {}, the address where the group \
                         begins (line {}).",
                        name, a, at, line
                    ),
                    (None, Some(a)) => format!("Variable `{}`: cell {:03}.", name, a),
                    _ => format!("`{}`", name),
                };
                if name == "S" || name == "X" {
                    let what = if name == "S" { "sum" } else { "product" };
                    s += &format!(
                        " Compiled code keeps the running {} of a formula without brackets here.",
                        what
                    );
                }
                s
            }
        },
        Kind::Other => "This is not part of the notation.".into(),
    }
}

/// Where each group begins: name → (line, address).
fn group_table(text: &str, unit: Option<&Unit>) -> BTreeMap<String, (usize, usize)> {
    let mut out = BTreeMap::new();
    match unit {
        Some(unit) => {
            for (item, (at, _)) in unit.items.iter().zip(placed(unit)) {
                if let Some(name) = label(&item.formula) {
                    out.insert(name.to_string(), (item.formula.line, at));
                }
            }
        }
        // A program that does not compile still has group headers to find.
        None => {
            for (n, line) in text.lines().enumerate() {
                if let [Token { kind: Kind::Name(p), .. }, Token { kind: Kind::Arrow, .. }, Token { kind: Kind::Name(name), .. }] =
                    tokens(line).as_slice()
                {
                    if p == "π′" {
                        out.insert(name.clone(), (n + 1, 0));
                    }
                }
            }
        }
    }
    out
}

fn token_at(line: &str, col: usize) -> Option<Token> {
    let t = tokens(line);
    t.iter()
        .find(|t| t.start <= col && col < t.end)
        .or(t.iter().find(|t| t.end == col))
        .cloned()
}

/// What to show when the pointer rests on `line` (from 0), column `col`:
/// the symbol there, and the instructions the formula on that line becomes.
pub fn hover(text: &str, line: usize, col: usize) -> Option<(String, Place)> {
    let src = text.lines().nth(line)?;
    let unit = Unit::build(text, Overrides::default()).ok();
    let groups = group_table(text, unit.as_ref());
    let mut parts = Vec::new();
    let (start, end) = extent(src);
    let mut place = (line, start, end);
    if let Some(t) = token_at(src, col) {
        parts.push(describe(&t, unit.as_ref(), &groups));
        place = (line, t.start, t.end);
    }
    if let Some(unit) = &unit {
        let mut listing = String::new();
        for (item, (at, words)) in unit.items.iter().zip(placed(unit)) {
            if item.formula.line != line + 1 {
                continue;
            }
            if item.label.is_some() {
                listing += &format!("the group begins at {:03}\n", at);
                continue;
            }
            for (i, w) in words.iter().enumerate() {
                listing +=
                    &format!("{:03}  {}   {}\n", at + i, show_word(*w), unit.layout.disasm(*w));
            }
        }
        if !listing.is_empty() {
            parts.push(format!("On the machine:\n\n```text\n{}```", listing));
        }
    }
    if parts.is_empty() {
        None
    } else {
        Some((parts.join("\n\n"), place))
    }
}

/// The kinds of token an editor colours, in the order the server declares
/// them. An editor with no grammar for the notation still gets colours.
pub const TOKEN_TYPES: [&str; 8] =
    ["comment", "string", "number", "variable", "function", "parameter", "operator", "keyword"];

/// Every token of a program with its kind: (line from 0, column, length,
/// index into `TOKEN_TYPES`), in order.
pub fn semantic(text: &str) -> Vec<(usize, usize, usize, usize)> {
    let groups = group_table(text, None);
    let mut out = Vec::new();
    for (n, line) in text.lines().enumerate() {
        for t in tokens(line) {
            let kind = match &t.kind {
                Kind::Num(_) => 2,
                Kind::Name(name) => match name.as_str() {
                    "?" => 5,
                    "π" | "π′" | "Ω" => 7,
                    name if groups.contains_key(name) => 4,
                    name if name.starts_with(|c: char| c.is_uppercase()) => 4,
                    _ => 3,
                },
                Kind::Op(_) | Kind::Arrow | Kind::Down | Kind::Open | Kind::Close => 6,
                Kind::Directive => 7,
                Kind::Other => continue,
            };
            out.push((n, t.start, t.end - t.start, kind));
        }
        if let Some((code, comment)) = line.split_once('#') {
            let start = code.encode_utf16().count();
            out.push((n, start, comment.encode_utf16().count() + 1, 0));
        }
    }
    out
}

/// Whether `name` could be written where a variable or group is expected.
pub fn is_name(name: &str) -> bool {
    matches!(tokens(name).as_slice(), [Token { kind: Kind::Name(n), .. }] if !special(n))
        && !name.contains(char::is_whitespace)
}

/// The name under the pointer, if it is one a program could define.
fn name_at(text: &str, line: usize, col: usize) -> Option<String> {
    match token_at(text.lines().nth(line)?, col)?.kind {
        Kind::Name(n) if !special(&n) => Some(n),
        _ => None,
    }
}

/// A place in a program: line from 0, and the columns of a token.
pub type Place = (usize, usize, usize);

/// The header of the group named under the pointer.
pub fn definition(text: &str, line: usize, col: usize) -> Option<Place> {
    let name = name_at(text, line, col)?;
    let (at, _) = *group_table(text, None).get(&name)?;
    let t = tokens(text.lines().nth(at - 1)?);
    let last = t.last()?;
    Some((at - 1, last.start, last.end))
}

/// Every use of the name under the pointer.
pub fn references(text: &str, line: usize, col: usize) -> Vec<Place> {
    let Some(name) = name_at(text, line, col) else { return Vec::new() };
    let mut out = Vec::new();
    for (n, src) in text.lines().enumerate() {
        for t in tokens(src) {
            if t.kind == Kind::Name(name.clone()) {
                out.push((n, t.start, t.end));
            }
        }
    }
    out
}

/// The groups of a program: name, the line of its header and its last line,
/// both from 0.
pub fn groups(text: &str) -> Vec<(String, usize, usize)> {
    let mut heads: Vec<(String, usize)> =
        group_table(text, None).into_iter().map(|(name, (line, _))| (name, line - 1)).collect();
    heads.sort_by_key(|h| h.1);
    let last = text.lines().count().saturating_sub(1);
    let mut out = Vec::new();
    for (i, (name, line)) in heads.iter().enumerate() {
        let mut end = heads.get(i + 1).map(|h| h.1 - 1).unwrap_or(last);
        while end > *line && tokens(text.lines().nth(end).unwrap_or("")).is_empty() {
            end -= 1;
        }
        out.push((name.clone(), *line, end));
    }
    out
}

/// The same program with every symbol written as the thesis prints it:
/// `->` becomes `→`, `pi` becomes `π`, `-` becomes `∸`, and so on. Comments,
/// spacing and directives are left alone.
pub fn symbolise(text: &str) -> String {
    let mut out = String::new();
    for line in text.split_inclusive('\n') {
        let body = line.strip_suffix('\n').unwrap_or(line);
        let (body, cr) = match body.strip_suffix('\r') {
            Some(b) => (b, "\r"),
            None => (body, ""),
        };
        if is_directive(body) {
            out += body;
        } else {
            let units: Vec<u16> = body.encode_utf16().collect();
            let mut at = 0;
            for t in tokens(body) {
                out += &String::from_utf16_lossy(&units[at..t.start]);
                at = t.end;
                let new = match &t.kind {
                    Kind::Arrow => "→".to_string(),
                    Kind::Down => "↓".to_string(),
                    Kind::Op(Op::Mod) => "mod".to_string(),
                    Kind::Op(op) => op.symbol().to_string(),
                    Kind::Name(n) if t.text.starts_with("pi") || t.text == "Omega" => {
                        // Keep the prime as typed: ' is what a keyboard has.
                        n.replace('′', "'")
                    }
                    _ => t.text.clone(),
                };
                out += &new;
            }
            out += &String::from_utf16_lossy(&units[at..]);
        }
        out += cr;
        if line.ends_with('\n') {
            out.push('\n');
        }
    }
    out
}

/// What can be typed, for completion: (what is inserted, how it is found
/// when typing on an ordinary keyboard, what it means).
pub const SYMBOLS: [(&str, &str, &str); 12] = [
    ("→", "->", "becomes"),
    ("π", "pi", "the group to continue at"),
    ("π'", "pi'", "the group that begins here"),
    ("Ω", "Omega", "no group: a cell that holds 0"),
    ("↓", "@", "the cell whose address is in the variable that follows"),
    ("∸", "-", "difference, or 0"),
    ("·", "*", "product"),
    (":", "/", "quotient"),
    ("÷", "~", "absolute difference"),
    ("∪", "max", "the greater"),
    ("∩", "min", "the smaller"),
    ("mod", "mod", "remainder"),
];

#[cfg(test)]
mod tests {
    use super::*;

    const TPK_LIKE: &str = "π' → A\n? → X\nX·X + 1 → S\nS → ?\nΩ → π\n";

    #[test]
    fn a_correct_program_has_no_problems() {
        let text = "π' → A\n? → n\n(1 ∩ n)·B → π\n\nπ' → B\nn → ?\n";
        assert_eq!(check(text), vec![]);
    }

    #[test]
    fn a_syntax_error_is_reported_on_its_line() {
        let d = check("π' → A\na mod 2 + 1 → ?\n");
        assert_eq!(d.len(), 1);
        assert!(d[0].error);
        assert_eq!(d[0].line, 2);
        assert!(d[0].message.contains("brackets"), "{}", d[0].message);
    }

    #[test]
    fn s_and_x_are_flagged_beside_a_polynomial() {
        let d = check(TPK_LIKE);
        let lines: Vec<(usize, bool)> = d.iter().map(|d| (d.line, d.error)).collect();
        assert_eq!(lines, vec![(2, false), (3, false), (3, false), (3, false), (4, false)]);
        // Without a polynomial formula there is nothing to collide with.
        assert_eq!(check("π' → A\n? → X\nX → ?\n"), vec![]);
    }

    #[test]
    fn a_jump_to_a_group_that_does_not_exist_is_flagged() {
        let d = check("π' → A\n? → n\n(1 ∩ n)·B + (1 ∸ n)·C → π\n\nπ' → B\nn → ?\n");
        assert_eq!(d.len(), 1);
        assert!(d[0].message.starts_with("C is never given a value"), "{}", d[0].message);
        // A return address in a variable is a value like any other.
        assert_eq!(check("π' → A\nB → z\nz → π\n\nπ' → B\n1 → ?\n"), vec![]);
    }

    #[test]
    fn hover_shows_the_instructions() {
        let text = "π' → A\n? → a\na + 1 → ?\n";
        let (h, place) = hover(text, 2, 2).unwrap();
        assert!(h.contains("`+` sum"), "{}", h);
        assert!(h.contains("501  "), "{}", h);
        assert_eq!(place, (2, 2, 3));
        let (h, _) = hover(text, 0, 5).unwrap();
        assert!(h.contains("Group `A`") && h.contains("begins at 500"), "{}", h);
    }

    #[test]
    fn groups_are_found_and_followed() {
        let text = "π' → A\nB → π\n\npi' -> B\n1 → ?\n";
        assert_eq!(definition(text, 1, 0), Some((3, 7, 8)));
        assert_eq!(references(text, 1, 0), vec![(1, 0, 1), (3, 7, 8)]);
        assert_eq!(groups(text), vec![("A".into(), 0, 1), ("B".into(), 3, 4)]);
    }

    #[test]
    fn every_token_has_a_kind() {
        let text = "π' → A   # start\n? → n\n(1 ∩ n)·B → π\nconst m = 3\n\nπ' → B\n";
        let kinds: Vec<(usize, usize, &str)> =
            semantic(text).into_iter().map(|(l, c, _, k)| (l, c, TOKEN_TYPES[k])).collect();
        assert_eq!(&kinds[..4], [(0, 0, "keyword"), (0, 3, "operator"), (0, 5, "function"), (0, 9, "comment")]);
        assert_eq!(&kinds[4..7], [(1, 0, "parameter"), (1, 2, "operator"), (1, 4, "variable")]);
        assert!(kinds.contains(&(2, 1, "number")) && kinds.contains(&(2, 8, "function")));
        assert!(kinds.contains(&(3, 0, "keyword")));
        assert!(is_name("m3") && is_name("A'") && !is_name("pi") && !is_name("ab") && !is_name("a b"));
    }

    #[test]
    fn symbolise_keeps_the_meaning() {
        let ascii = "pi' -> A   # a - b stays\n? -> a\n(a - 1)*2 / 3 ~ @a -> ?\na max 1 -> pi\nconst m = 3\nOmega -> pi\n";
        let pretty = symbolise(ascii);
        assert_eq!(
            pretty,
            "π' → A   # a - b stays\n? → a\n(a ∸ 1)·2 : 3 ÷ ↓a → ?\na ∪ 1 → π\nconst m = 3\nΩ → π\n"
        );
        assert_eq!(symbolise(&pretty), pretty);
    }

    #[test]
    fn every_example_is_clean() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples");
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().is_some_and(|e| e == "bohm") {
                let text = std::fs::read_to_string(&path).unwrap();
                assert_eq!(check(&text), vec![], "{}", path.display());
                assert_eq!(symbolise(&text), text, "{}", path.display());
            }
        }
    }
}
