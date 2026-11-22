//! The notation of chapters 2 and 3, as text.
//!
//! Unicode symbols are accepted as printed in the thesis, each with an ASCII
//! spelling:
//!
//! | thesis | ASCII   | meaning                         |
//! |--------|---------|---------------------------------|
//! | `→`    | `->`    | becomes                         |
//! | `↓x`   | `@x`    | the quantity whose index is x   |
//! | `π`    | `pi`    | the group to choose next        |
//! | `π′`   | `pi'`   | the group that begins here      |
//! | `Ω`    | `Omega` | no group (a cell holding 0)     |
//! | `∸`    | `-`     | difference, or 0 if negative    |
//! | `·`    | `*`     | product                         |
//! | `:`    | `/`     | integer quotient                |
//! | `÷`    | `~`     | absolute difference             |
//! | `∪`    | `max`   | the greater                     |
//! | `∩`    | `min`   | the smaller                     |
//!
//! Round, square and curly brackets are interchangeable. A numeral stands for
//! a cell that holds that number (the thesis prints these in italics). A name
//! is one letter, optionally followed by an index (`m3`) and primes (`A'`).
//! `#` starts a comment. Three directives set up what the thesis leaves to the
//! operator: `const NAME = N`, `origin N`, `temps N`, `cells N`.

use crate::machine::Op;

#[derive(Debug, Clone, PartialEq)]
pub struct Cell {
    pub name: String,
    pub ind: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Cell(Cell),
    Num(u64),
    Bin(Box<Expr>, Op, Box<Expr>),
    Paren(Box<Expr>),
}

#[derive(Debug, Clone)]
pub struct Formula {
    pub src: Expr,
    pub dst: Cell,
    pub line: usize,
}

#[derive(Debug, Clone, Default)]
pub struct Source {
    pub formulas: Vec<Formula>,
    pub consts: Vec<(String, u64)>,
    pub origin: Option<usize>,
    pub temps: Option<usize>,
    pub cells: Option<usize>,
}

#[derive(Debug, Clone, PartialEq)]
enum Tok {
    Name(String),
    Num(u64),
    Op(Op),
    Arrow,
    Down,
    Open,
    Close,
}

fn err<T>(line: usize, msg: impl Into<String>) -> Result<T, String> {
    Err(format!("line {}: {}", line, msg.into()))
}

fn lex(text: &str, line: usize, out: &mut Vec<(Tok, usize)>) -> Result<(), String> {
    let cs: Vec<char> = text.chars().collect();
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
        if c.is_whitespace() {
            i += 1;
        } else if c.is_ascii_digit() {
            let mut v: u64 = 0;
            while i < cs.len() && cs[i].is_ascii_digit() {
                v = v
                    .checked_mul(10)
                    .and_then(|v| v.checked_add(cs[i] as u64 - '0' as u64))
                    .filter(|v| *v < crate::machine::WORD)
                    .ok_or(format!("line {}: number longer than 14 digits", line))?;
                i += 1;
            }
            out.push((Tok::Num(v), line));
        } else if c.is_ascii_alphabetic() {
            let mut word = String::new();
            while i < cs.len() && cs[i].is_ascii_alphabetic() {
                word.push(cs[i]);
                i += 1;
            }
            let tok = match word.as_str() {
                "mod" => Tok::Op(Op::Mod),
                "max" => Tok::Op(Op::Max),
                "min" => Tok::Op(Op::Min),
                "pi" => {
                    let mut n = "π".to_string();
                    suffix(&mut i, &mut n);
                    Tok::Name(n)
                }
                "Omega" => Tok::Name("Ω".into()),
                w if w.len() == 1 => {
                    let mut n = w.to_string();
                    suffix(&mut i, &mut n);
                    Tok::Name(n)
                }
                w => return err(line, format!("`{}` is not a name: names are single letters", w)),
            };
            out.push((tok, line));
        } else {
            i += 1;
            let tok = match c {
                '→' => Tok::Arrow,
                '-' if i < cs.len() && cs[i] == '>' => {
                    i += 1;
                    Tok::Arrow
                }
                '↓' | '@' => Tok::Down,
                '(' | '[' | '{' => Tok::Open,
                ')' | ']' | '}' => Tok::Close,
                '+' => Tok::Op(Op::Add),
                '∸' | '-' | '−' => Tok::Op(Op::Sub),
                '·' | '*' => Tok::Op(Op::Mul),
                ':' | '/' => Tok::Op(Op::Div),
                '÷' | '~' => Tok::Op(Op::Diff),
                '∪' => Tok::Op(Op::Max),
                '∩' => Tok::Op(Op::Min),
                '?' => Tok::Name("?".into()),
                c if c.is_alphabetic() => {
                    let mut n = c.to_string();
                    suffix(&mut i, &mut n);
                    Tok::Name(n)
                }
                c => return err(line, format!("unexpected character `{}`", c)),
            };
            out.push((tok, line));
        }
    }
    Ok(())
}

struct Parser {
    toks: Vec<(Tok, usize)>,
    i: usize,
}

fn loose(op: Op) -> bool {
    matches!(op, Op::Add | Op::Sub)
}
fn tight(op: Op) -> bool {
    matches!(op, Op::Mul | Op::Div)
}

impl Parser {
    fn peek(&self) -> Option<&Tok> {
        self.toks.get(self.i).map(|t| &t.0)
    }
    fn line(&self) -> usize {
        self.toks.get(self.i).or(self.toks.last()).map(|t| t.1).unwrap_or(0)
    }
    fn peek_op(&self, f: fn(Op) -> bool) -> Option<Op> {
        match self.peek() {
            Some(Tok::Op(op)) if f(*op) => Some(*op),
            _ => None,
        }
    }

    /// The thesis only ever writes ÷, mod, ∪ and ∩ between two operands or
    /// inside brackets, so their rank against + and · is never stated.
    fn unranked(&self, e: &Expr) -> Result<(), String> {
        match e {
            Expr::Bin(_, op, _) if !loose(*op) && !tight(*op) => err(
                self.line(),
                format!("put brackets around the `{}` operation", op.symbol().trim()),
            ),
            _ => Ok(()),
        }
    }

    fn expr(&mut self) -> Result<Expr, String> {
        let mut l = self.term()?;
        while let Some(op) = self.peek_op(loose) {
            self.unranked(&l)?;
            self.i += 1;
            let r = self.term()?;
            self.unranked(&r)?;
            l = Expr::Bin(Box::new(l), op, Box::new(r));
        }
        Ok(l)
    }

    fn term(&mut self) -> Result<Expr, String> {
        let mut l = self.factor()?;
        while let Some(op) = self.peek_op(tight) {
            self.unranked(&l)?;
            self.i += 1;
            let r = self.factor()?;
            self.unranked(&r)?;
            l = Expr::Bin(Box::new(l), op, Box::new(r));
        }
        Ok(l)
    }

    fn factor(&mut self) -> Result<Expr, String> {
        let l = self.primary()?;
        if let Some(op) = self.peek_op(|op| !loose(op) && !tight(op)) {
            self.i += 1;
            let r = self.primary()?;
            if self.peek_op(|op| !loose(op) && !tight(op)).is_some() {
                return err(self.line(), "put brackets around each ÷, mod, ∪ or ∩ operation");
            }
            return Ok(Expr::Bin(Box::new(l), op, Box::new(r)));
        }
        Ok(l)
    }

    fn primary(&mut self) -> Result<Expr, String> {
        let line = self.line();
        let tok = self.peek().cloned();
        self.i += 1;
        match tok {
            Some(Tok::Open) => {
                let e = self.expr()?;
                if self.peek() != Some(&Tok::Close) {
                    return err(self.line(), "missing closing bracket");
                }
                self.i += 1;
                Ok(Expr::Paren(Box::new(e)))
            }
            Some(Tok::Down) => match self.peek().cloned() {
                Some(Tok::Name(name)) => {
                    self.i += 1;
                    Ok(Expr::Cell(Cell { name, ind: true }))
                }
                _ => err(line, "↓ must be followed by a name"),
            },
            Some(Tok::Name(name)) => Ok(Expr::Cell(Cell { name, ind: false })),
            Some(Tok::Num(v)) => Ok(Expr::Num(v)),
            _ => err(line, "expected a variable, a number or a bracket"),
        }
    }

    fn formula(&mut self) -> Result<Formula, String> {
        let line = self.line();
        let src = self.expr()?;
        if self.peek() != Some(&Tok::Arrow) {
            return err(self.line(), "expected →");
        }
        self.i += 1;
        let ind = self.peek() == Some(&Tok::Down);
        if ind {
            self.i += 1;
        }
        match self.peek().cloned() {
            Some(Tok::Name(name)) => {
                self.i += 1;
                Ok(Formula { src, dst: Cell { name, ind }, line })
            }
            _ => err(self.line(), "a formula must end with a single variable after →"),
        }
    }
}

pub fn parse(text: &str) -> Result<Source, String> {
    let mut src = Source::default();
    let mut toks = Vec::new();
    for (n, raw) in text.lines().enumerate() {
        let line = n + 1;
        let code = raw.split('#').next().unwrap().trim();
        let mut words = code.split_whitespace();
        let number = |w: Option<&str>| -> Result<u64, String> {
            w.and_then(|w| w.parse::<u64>().ok())
                .filter(|v| *v < crate::machine::WORD)
                .ok_or(format!("line {}: expected a number", line))
        };
        match words.next() {
            Some("const") => {
                let rest = code["const".len()..].trim();
                let (name, value) = rest
                    .split_once('=')
                    .ok_or(format!("line {}: expected `const NAME = N`", line))?;
                let mut t = Vec::new();
                lex(name, line, &mut t)?;
                match t.as_slice() {
                    [(Tok::Name(n), _)] => {
                        src.consts.push((n.clone(), number(Some(value.trim()))?))
                    }
                    _ => return err(line, "expected `const NAME = N`"),
                }
            }
            Some("origin") => src.origin = Some(number(words.next())? as usize),
            Some("temps") => src.temps = Some(number(words.next())? as usize),
            Some("cells") => src.cells = Some(number(words.next())? as usize),
            _ => lex(code, line, &mut toks)?,
        }
    }
    let mut p = Parser { toks, i: 0 };
    while p.peek().is_some() {
        src.formulas.push(p.formula()?);
    }
    Ok(src)
}
