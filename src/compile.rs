//! The reference translator: chapters 4.3, 5 and 6 written directly in Rust.
//!
//! This is not a transliteration of Böhm's chapter 7 program. It is a second,
//! independent reading of the method that program implements, so the two can
//! be compared number for number.
//!
//! A source formula is first put into one of the three shapes Böhm's compiler
//! accepts:
//!
//! * a store, `a → b`;
//! * a polynomial in normal form (chapter 6): no brackets, only + ∸ · :
//! * a fully bracketed formula of binary operations (chapter 5).
//!
//! The thesis itself writes formulas that are in none of these shapes (its own
//! compiler listing mixes brackets with precedence, see footnote 18), so the
//! third shape is produced by bracketing every operation.

use crate::machine::{ins, Op, CELLS, OMEGA, PI_PRIME, TRANSFER};
use crate::syntax::{self, Cell, Expr, Formula, Source};
use std::collections::HashMap;

/// The letters Böhm's compiler uses for intermediate results of a polynomial.
pub const S: u64 = 46;
pub const X: u64 = 51;

pub const DEFAULT_CELLS: usize = 54;
pub const DEFAULT_TEMPS: usize = 80;
pub const DEFAULT_ORIGIN: usize = 500;

/// 4.3: the number the teletypewriter records for each key.
pub const CLOSE: u64 = 30;
pub const OPEN: u64 = 31;
pub const ARROW: u64 = 27;
pub fn var_code(k: u64) -> u64 {
    5 * k + 3
}
pub fn op_code(op: Op) -> u64 {
    5 * op.code() + 4
}

/// Where every name lives. Letters are fixed by convention 2.4; everything
/// else (primed and indexed letters, Greek letters, numerals) gets a cell
/// from `cells` upward, in order of first appearance.
#[derive(Debug, Clone)]
pub struct Layout {
    names: HashMap<String, usize>,
    rev: HashMap<usize, String>,
    pub cells: usize,
    pub next: usize,
    pub temps: usize,
    pub origin: usize,
    pub consts: Vec<(usize, u64)>,
}

fn fixed(name: &str) -> Option<usize> {
    match name {
        "π" => return Some(0),
        "?" => return Some(1),
        "Ω" => return Some(OMEGA),
        "π′" => return Some(PI_PRIME),
        _ => {}
    }
    let mut cs = name.chars();
    match (cs.next(), cs.next()) {
        (Some(c), None) if c.is_ascii_lowercase() => Some(2 + (c as usize - 'a' as usize)),
        (Some(c), None) if c.is_ascii_uppercase() => Some(28 + (c as usize - 'A' as usize)),
        _ => None,
    }
}

impl Layout {
    fn new(cells: usize, temps: usize, origin: usize) -> Layout {
        Layout {
            names: HashMap::new(),
            rev: HashMap::new(),
            cells,
            next: cells,
            temps,
            origin,
            consts: Vec::new(),
        }
    }

    pub fn addr(&mut self, name: &str) -> usize {
        if let Some(a) = fixed(name) {
            return a;
        }
        if let Some(a) = self.names.get(name) {
            return *a;
        }
        let a = self.next;
        self.next += 1;
        self.names.insert(name.to_string(), a);
        self.rev.insert(a, name.to_string());
        a
    }

    pub fn lookup(&self, name: &str) -> Option<usize> {
        fixed(name).or(self.names.get(name).copied())
    }

    /// 2.6: a numeral is the address of a cell containing that number.
    /// Zero is Ω, as in the thesis (`0 → π` and `Ω → π` are the same).
    fn num(&mut self, v: u64) -> usize {
        if v == 0 {
            return OMEGA;
        }
        let name = v.to_string();
        if let Some(a) = self.names.get(&name) {
            return *a;
        }
        let a = self.addr(&name);
        self.consts.push((a, v));
        a
    }

    fn cell(&mut self, c: &Cell) -> u64 {
        self.addr(&c.name) as u64 + if c.ind { 1000 } else { 0 }
    }

    pub fn name_of(&self, addr: usize) -> String {
        match addr {
            0 => "π".into(),
            1 => "?".into(),
            OMEGA => "Ω".into(),
            PI_PRIME => "π′".into(),
            2..=27 => ((b'a' + (addr - 2) as u8) as char).to_string(),
            28..=53 => ((b'A' + (addr - 28) as u8) as char).to_string(),
            _ => match self.rev.get(&addr) {
                Some(n) if n.chars().next().unwrap().is_ascii_digit() => format!("‹{}›", n),
                Some(n) => n.clone(),
                None => format!("[{:03}]", addr),
            },
        }
    }

    pub fn show(&self, k: u64) -> String {
        let name = self.name_of((k % 1000) as usize);
        if k >= 1000 {
            format!("↓{}", name)
        } else {
            name
        }
    }

    /// An instruction number back in the notation of chapter 2.
    pub fn disasm(&self, word: u64) -> String {
        use crate::machine::{M1, M2, M3};
        let (k1, cop, k2, kr) = (word / M1, word / M2 % 100, word / M3 % 10_000, word % 10_000);
        if word == 0 {
            "stop".into()
        } else if cop == TRANSFER {
            format!("{} → {}", self.show(k2), self.show(kr))
        } else if let Some(op) = Op::from_code(cop) {
            format!("{}{}{} → {}", self.show(k1), op.symbol(), self.show(k2), self.show(kr))
        } else {
            "(not an instruction)".into()
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Mono {
    pub head: u64,
    pub tail: Vec<(Op, u64)>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Tree {
    Leaf(u64),
    Node(Box<Tree>, Op, Box<Tree>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Normal {
    Store { src: u64, dst: u64 },
    Poly { first: Mono, rest: Vec<(Op, Mono)>, dst: u64 },
    Paren { tree: Tree, dst: u64 },
}

fn strip(e: &Expr) -> &Expr {
    match e {
        Expr::Paren(inner) => strip(inner),
        e => e,
    }
}

fn plain_polynomial(e: &Expr) -> bool {
    match e {
        Expr::Cell(_) | Expr::Num(_) => true,
        Expr::Paren(_) => false,
        Expr::Bin(l, op, r) => {
            matches!(op, Op::Add | Op::Sub | Op::Mul | Op::Div)
                && plain_polynomial(l)
                && plain_polynomial(r)
        }
    }
}

impl Layout {
    fn atom(&mut self, e: &Expr) -> u64 {
        match e {
            Expr::Cell(c) => self.cell(c),
            Expr::Num(v) => self.num(*v) as u64,
            _ => unreachable!("not an operand"),
        }
    }

    fn mono(&mut self, e: &Expr) -> Mono {
        match e {
            Expr::Bin(l, op, r) => {
                let mut m = self.mono(l);
                let k = self.atom(r);
                m.tail.push((*op, k));
                m
            }
            e => Mono { head: self.atom(e), tail: Vec::new() },
        }
    }

    fn monos(&mut self, e: &Expr, out: &mut Vec<(Op, Mono)>) {
        match e {
            Expr::Bin(l, op, r) if matches!(op, Op::Add | Op::Sub) => {
                self.monos(l, out);
                let m = self.mono(r);
                out.push((*op, m));
            }
            e => {
                let m = self.mono(e);
                out.push((Op::Add, m));
            }
        }
    }

    fn tree(&mut self, e: &Expr) -> Tree {
        match strip(e) {
            Expr::Bin(l, op, r) => {
                let l = self.tree(l);
                let r = self.tree(r);
                Tree::Node(Box::new(l), *op, Box::new(r))
            }
            e => Tree::Leaf(self.atom(e)),
        }
    }

    fn normal(&mut self, f: &Formula) -> Normal {
        let e = strip(&f.src);
        if matches!(e, Expr::Cell(_) | Expr::Num(_)) {
            let src = self.atom(e);
            Normal::Store { src, dst: self.cell(&f.dst) }
        } else if plain_polynomial(&f.src) {
            let mut all = Vec::new();
            self.monos(&f.src, &mut all);
            let first = all.remove(0).1;
            Normal::Poly { first, rest: all, dst: self.cell(&f.dst) }
        } else {
            let tree = self.tree(e);
            Normal::Paren { tree, dst: self.cell(&f.dst) }
        }
    }
}

impl Tree {
    pub fn regions(&self) -> usize {
        match self {
            Tree::Leaf(_) => 0,
            Tree::Node(l, _, r) => 1 + l.regions() + r.regions(),
        }
    }

    fn chars(&self, out: &mut Vec<u64>) {
        match self {
            Tree::Leaf(k) => out.push(var_code(*k)),
            Tree::Node(l, op, r) => {
                out.push(OPEN);
                l.chars(out);
                out.push(op_code(*op));
                r.chars(out);
                out.push(CLOSE);
            }
        }
    }

    fn show(&self, lay: &Layout) -> String {
        match self {
            Tree::Leaf(k) => lay.show(*k),
            Tree::Node(l, op, r) => format!("({}{}{})", l.show(lay), op.symbol(), r.show(lay)),
        }
    }

    /// 5.3 and 5.4: every bracket pair is a region, numbered in the order the
    /// opening brackets are met. Region f computes into the cell m+f.
    fn code(&self, m: u64, next: &mut u64, out: &mut Vec<(u64, u64)>) -> u64 {
        match self {
            Tree::Leaf(k) => *k,
            Tree::Node(l, op, r) => {
                *next += 1;
                let f = *next;
                let a = l.code(m, next, out);
                let b = r.code(m, next, out);
                out.push((f, ins(a, op.code(), b, m + f)));
                m + f
            }
        }
    }
}

impl Normal {
    /// 4.3: the formula as the teletypewriter would record it.
    pub fn chars(&self, out: &mut Vec<u64>) {
        match self {
            Normal::Store { src, dst } => out.extend([var_code(*src), ARROW, var_code(*dst)]),
            Normal::Poly { first, rest, dst } => {
                let mono = |m: &Mono, out: &mut Vec<u64>| {
                    out.push(var_code(m.head));
                    for (op, k) in &m.tail {
                        out.extend([op_code(*op), var_code(*k)]);
                    }
                };
                mono(first, out);
                for (op, m) in rest {
                    out.push(op_code(*op));
                    mono(m, out);
                }
                out.extend([ARROW, var_code(*dst)]);
            }
            Normal::Paren { tree, dst } => {
                tree.chars(out);
                out.extend([ARROW, var_code(*dst)]);
            }
        }
    }

    pub fn show(&self, lay: &Layout) -> String {
        match self {
            Normal::Store { src, dst } => format!("{} → {}", lay.show(*src), lay.show(*dst)),
            Normal::Poly { first, rest, dst } => {
                let mono = |m: &Mono| {
                    let mut s = lay.show(m.head);
                    for (op, k) in &m.tail {
                        s += op.symbol();
                        s += &lay.show(*k);
                    }
                    s
                };
                let mut s = mono(first);
                for (op, m) in rest {
                    s += op.symbol();
                    s += &mono(m);
                }
                format!("{} → {}", s, lay.show(*dst))
            }
            Normal::Paren { tree, dst } => format!("{} → {}", tree.show(lay), lay.show(*dst)),
        }
    }

    pub fn regions(&self) -> usize {
        match self {
            Normal::Paren { tree, .. } => tree.regions(),
            _ => 0,
        }
    }

    /// The instruction numbers chapters 5 and 6 prescribe. `m` is the constant
    /// of the compiler: intermediate results of bracketed formulas go to m+1, m+2, ...
    pub fn code(&self, m: u64, out: &mut Vec<u64>) {
        match self {
            Normal::Store { src, dst } => out.push(ins(0, TRANSFER, *src, *dst)),
            Normal::Paren { tree, dst } => {
                // 5.3: instruction numbers come out in decreasing order of region.
                let mut regions = Vec::new();
                tree.code(m, &mut 0, &mut regions);
                regions.sort_by_key(|r| std::cmp::Reverse(r.0));
                out.extend(regions.iter().map(|r| r.1));
                out.push(ins(0, TRANSFER, m + 1, *dst));
            }
            Normal::Poly { first, rest, dst } => {
                // Table (6.2). S collects the sum, X the current product.
                let product = |m: &Mono, out: &mut Vec<u64>| {
                    out.push(ins(0, TRANSFER, m.head, X));
                    for (op, k) in &m.tail {
                        out.push(ins(X, op.code(), *k, X));
                    }
                };
                if first.tail.is_empty() {
                    out.push(ins(0, TRANSFER, first.head, S));
                } else {
                    out.push(ins(0, TRANSFER, OMEGA as u64, S));
                    product(first, out);
                    out.push(ins(S, Op::Add.code(), X, S));
                }
                for (sign, mono) in rest {
                    if mono.tail.is_empty() {
                        out.push(ins(S, sign.code(), mono.head, S));
                    } else {
                        product(mono, out);
                        out.push(ins(S, sign.code(), X, S));
                    }
                }
                out.push(ins(0, TRANSFER, S, *dst));
            }
        }
    }
}

/// One formula of a program, in every form the tools need.
#[derive(Debug, Clone)]
pub struct Item {
    pub formula: Formula,
    pub normal: Normal,
    /// `π′ → K`: this formula names the group that begins here.
    pub label: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct Unit {
    pub layout: Layout,
    pub items: Vec<Item>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Overrides {
    pub temps: Option<usize>,
    pub origin: Option<usize>,
}

impl Unit {
    pub fn build(text: &str, over: Overrides) -> Result<Unit, String> {
        let src: Source = syntax::parse(text)?;
        let mut layout = Layout::new(
            src.cells.unwrap_or(DEFAULT_CELLS),
            over.temps.or(src.temps).unwrap_or(DEFAULT_TEMPS),
            over.origin.or(src.origin).unwrap_or(DEFAULT_ORIGIN),
        );
        for (name, value) in &src.consts {
            let a = layout.addr(name);
            layout.consts.push((a, *value));
        }
        let mut items = Vec::new();
        for f in &src.formulas {
            let normal = layout.normal(f);
            let label = match normal {
                Normal::Store { src, dst } if src == PI_PRIME as u64 && dst < 1000 => {
                    Some(dst as usize)
                }
                _ => None,
            };
            items.push(Item { formula: f.clone(), normal, label });
        }
        let unit = Unit { layout, items };
        unit.check()?;
        Ok(unit)
    }

    /// The compiler constant m this program is compiled with.
    pub fn m(&self) -> u64 {
        self.layout.temps as u64
    }

    pub fn max_regions(&self) -> usize {
        self.items.iter().map(|i| i.normal.regions()).max().unwrap_or(0)
    }

    /// The relocatable instruction numbers, group headers included.
    pub fn code(&self) -> Vec<u64> {
        let mut out = Vec::new();
        for item in &self.items {
            item.normal.code(self.m(), &mut out);
        }
        out
    }

    /// The whole program as recorded by the teletypewriter, ended by a
    /// closing bracket (remark in 6.3).
    pub fn chars(&self) -> Vec<u64> {
        let mut out = Vec::new();
        for item in &self.items {
            item.normal.chars(&mut out);
        }
        out.push(CLOSE);
        out
    }

    /// Number of memory cells the loaded program occupies.
    pub fn size(&self) -> usize {
        let mut out = Vec::new();
        for item in self.items.iter().filter(|i| i.label.is_none()) {
            item.normal.code(self.m(), &mut out);
        }
        out.len()
    }

    /// What must be in memory before the program is loaded: the constants,
    /// and π′ pointing at the first free address.
    pub fn prefix(&self) -> Vec<(usize, u64)> {
        let mut p = self.layout.consts.clone();
        p.push((PI_PRIME, self.layout.origin as u64));
        p
    }

    /// The cell of the first group, where the operator starts the machine.
    pub fn start(&self) -> Result<usize, String> {
        self.items
            .iter()
            .find_map(|i| i.label)
            .ok_or("the program has no group (no `π′ → A`)".to_string())
    }

    /// Memory regions as (name, first, last).
    pub fn regions(&self) -> Vec<(&'static str, usize, usize)> {
        let l = &self.layout;
        let mut r = Vec::new();
        if l.next > l.cells {
            r.push(("named cells", l.cells, l.next - 1));
        }
        if self.max_regions() > 0 {
            r.push(("intermediate results", l.temps + 1, l.temps + self.max_regions()));
        }
        if self.size() > 0 {
            r.push(("instructions", l.origin, l.origin + self.size() - 1));
        }
        r
    }

    fn check(&self) -> Result<(), String> {
        let regions = self.regions();
        for (i, a) in regions.iter().enumerate() {
            if a.1 < 54 || a.2 >= CELLS - 2 {
                return Err(format!(
                    "{} ({}..{}) fall outside the free memory 054..997; set `cells`, `temps` or `origin`",
                    a.0, a.1, a.2
                ));
            }
            for b in &regions[i + 1..] {
                if a.1 <= b.2 && b.1 <= a.2 {
                    return Err(format!(
                        "{} ({}..{}) overlap {} ({}..{}); set `cells`, `temps` or `origin`",
                        a.0, a.1, a.2, b.0, b.1, b.2
                    ));
                }
            }
        }
        Ok(())
    }
}
