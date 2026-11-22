//! The second interpretation of the notation (chapter 3): formulas read as
//! logic and algebra, with no instruction numbers and no intermediate cells.
//!
//! A group name is a positive integer (3.33): here, the position of the
//! group's first formula. π holds the position of the next formula. Each
//! formula is one step, whatever its size.
//!
//! The thesis does not say in what order a formula reads its operands, which
//! matters only when it reads `?` more than once. Here it is the order of the
//! instructions chapters 5 and 6 write for the formula: a polynomial left to
//! right, a bracketed formula in the order of (5.32) (see `eval_bracketed`).
//! That rule is ours, chosen so that the readings agree.

use crate::compile::{Normal, Unit};
use crate::machine::{Fault, CELLS, IO, PI};
use crate::syntax::{Cell, Expr};
use std::collections::VecDeque;

fn is_operation(e: &Expr) -> bool {
    match e {
        Expr::Paren(inner) => is_operation(inner),
        Expr::Bin(..) => true,
        _ => false,
    }
}

pub struct Interp<'a> {
    unit: &'a Unit,
    /// Indexes into unit.items of the formulas that are executed.
    steps_of: Vec<usize>,
    pub mem: Vec<u64>,
    pub input: VecDeque<u64>,
    pub output: Vec<u64>,
    pub steps: u64,
}

impl<'a> Interp<'a> {
    pub fn new(unit: &'a Unit, input: &[u64]) -> Result<Interp<'a>, String> {
        let mut mem = vec![0; CELLS];
        for (a, v) in &unit.layout.consts {
            mem[*a] = *v;
        }
        let mut steps_of = Vec::new();
        for (i, item) in unit.items.iter().enumerate() {
            match item.label {
                Some(cell) => mem[cell] = steps_of.len() as u64 + 1,
                None => steps_of.push(i),
            }
        }
        mem[PI] = mem[unit.start()?];
        Ok(Interp {
            unit,
            steps_of,
            mem,
            input: input.iter().copied().collect(),
            output: Vec::new(),
            steps: 0,
        })
    }

    fn load(&mut self, a: usize) -> Result<u64, Fault> {
        if a == IO {
            self.input.pop_front().ok_or(Fault::InputExhausted)
        } else {
            Ok(self.mem[a])
        }
    }

    fn address(&mut self, c: &Cell) -> Result<usize, Fault> {
        let a = self.unit.layout.lookup(&c.name).expect("name was laid out");
        if c.ind {
            Ok((self.load(a)? % 1000) as usize)
        } else {
            Ok(a)
        }
    }

    fn eval(&mut self, e: &Expr) -> Result<u64, Fault> {
        match e {
            Expr::Num(v) => Ok(*v),
            Expr::Paren(e) => self.eval(e),
            Expr::Cell(c) => {
                let a = self.address(c)?;
                self.load(a)
            }
            Expr::Bin(l, op, r) => {
                let a = self.eval(l)?;
                let b = self.eval(r)?;
                op.apply(a, b)
            }
        }
    }

    /// A bracketed formula, operands read in the order the machine reads
    /// them. Chapter 5 numbers the operations in the order their opening
    /// brackets are met and writes them in decreasing order (5.32), so the
    /// right operand's operations come before the left's, and both before
    /// the operation itself, which reads its own plain operands left first.
    fn eval_bracketed(&mut self, e: &Expr) -> Result<u64, Fault> {
        match e {
            Expr::Paren(inner) => self.eval_bracketed(inner),
            Expr::Bin(l, op, r) => {
                let rv = if is_operation(r) { Some(self.eval_bracketed(r)?) } else { None };
                let lv = if is_operation(l) { Some(self.eval_bracketed(l)?) } else { None };
                let a = match lv {
                    Some(v) => v,
                    None => self.eval(l)?,
                };
                let b = match rv {
                    Some(v) => v,
                    None => self.eval(r)?,
                };
                op.apply(a, b)
            }
            e => self.eval(e),
        }
    }

    pub fn run(&mut self, limit: u64) -> Result<(), Fault> {
        loop {
            let pc = self.mem[PI];
            // Past the last formula there is an empty cell, which stops the machine.
            if pc == 0 || pc as usize == self.steps_of.len() + 1 {
                return Ok(());
            }
            if pc as usize > self.steps_of.len() {
                return Err(Fault::PiOutOfRange(pc));
            }
            if self.steps >= limit {
                return Err(Fault::StepLimit);
            }
            self.steps += 1;
            self.mem[PI] = pc + 1;
            let unit = self.unit;
            let item = &unit.items[self.steps_of[pc as usize - 1]];
            let f = &item.formula;
            let v = match item.normal {
                Normal::Paren { .. } => self.eval_bracketed(&f.src)?,
                _ => self.eval(&f.src)?,
            };
            match self.address(&f.dst)? {
                PI => self.mem[PI] = v % 10_000,
                IO => self.output.push(v),
                a => self.mem[a] = v,
            }
        }
    }
}
