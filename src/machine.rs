//! Böhm's three-address machine (thesis chapter 1).
//!
//! * about 1000 cells, each holding a non-negative integer of 14 decimal digits;
//! * address 000 is the register π (four digits), address 001 is both the
//!   reader and the printer;
//! * an instruction is the number `ε1 c1 cop ε2 c2 εr cr` (1+3+2+1+3+1+3 digits);
//! * the fundamental cycle is: load from the address in π, increment π, execute.

use std::collections::VecDeque;
use std::fmt;

pub const WORD: u64 = 100_000_000_000_000; // 10^14
pub const M1: u64 = 10_000_000_000; // 10^10, position of the first operand
pub const M2: u64 = 100_000_000; // 10^8, position of the operation code
pub const M3: u64 = 10_000; // 10^4, position of the second operand

pub const PI: usize = 0;
pub const IO: usize = 1;
pub const OMEGA: usize = 998;
pub const PI_PRIME: usize = 999;
pub const CELLS: usize = 1000;

/// Operation code 05: transfer.
pub const TRANSFER: u64 = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Op {
    Add,
    Mul,
    Sub,
    Div,
    Diff,
    Mod,
    Max,
    Min,
}

impl Op {
    /// The table of section 1.3.
    pub fn code(self) -> u64 {
        match self {
            Op::Add => 1,
            Op::Mul => 2,
            Op::Sub => 3,
            Op::Div => 4,
            Op::Diff => 6,
            Op::Mod => 7,
            Op::Max => 8,
            Op::Min => 9,
        }
    }

    pub fn from_code(c: u64) -> Option<Op> {
        Some(match c {
            1 => Op::Add,
            2 => Op::Mul,
            3 => Op::Sub,
            4 => Op::Div,
            6 => Op::Diff,
            7 => Op::Mod,
            8 => Op::Max,
            9 => Op::Min,
            _ => return None,
        })
    }

    pub fn symbol(self) -> &'static str {
        match self {
            Op::Add => "+",
            Op::Mul => "·",
            Op::Sub => "∸",
            Op::Div => ":",
            Op::Diff => "÷",
            Op::Mod => " mod ",
            Op::Max => "∪",
            Op::Min => "∩",
        }
    }

    pub fn apply(self, a: u64, b: u64) -> Result<u64, Fault> {
        Ok(match self {
            Op::Add => (a + b) % WORD,
            Op::Mul => ((a as u128 * b as u128) % WORD as u128) as u64,
            Op::Sub => a.saturating_sub(b),
            Op::Div => {
                if b == 0 {
                    return Err(Fault::DivideByZero);
                }
                a / b
            }
            Op::Diff => a.abs_diff(b),
            Op::Mod => {
                if b == 0 {
                    return Err(Fault::DivideByZero);
                }
                a % b
            }
            Op::Max => a.max(b),
            Op::Min => a.min(b),
        })
    }
}

/// Things the thesis leaves undefined. The simulator stops and reports them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Fault {
    InputExhausted,
    DivideByZero,
    BadInstruction { at: usize, word: u64 },
    PiOutOfRange(u64),
    StepLimit,
}

impl fmt::Display for Fault {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Fault::InputExhausted => write!(f, "input tape exhausted"),
            Fault::DivideByZero => write!(f, "division by zero"),
            Fault::BadInstruction { at, word } => {
                write!(f, "undefined instruction {} at address {:03}", show_word(*word), at)
            }
            Fault::PiOutOfRange(p) => write!(f, "π = {} is outside the memory", p),
            Fault::StepLimit => write!(f, "step limit reached"),
        }
    }
}

/// `0051 02 0017 0051`
pub fn show_word(n: u64) -> String {
    format!("{:04} {:02} {:04} {:04}", n / M1, n / M2 % 100, n / M3 % 10_000, n % 10_000)
}

/// Build an instruction number. Each `k` is an address plus 1000 for ↓.
pub fn ins(k1: u64, cop: u64, k2: u64, kr: u64) -> u64 {
    k1 * M1 + cop * M2 + k2 * M3 + kr
}

pub struct Machine {
    pub mem: Vec<u64>,
    pub input: VecDeque<u64>,
    pub output: Vec<u64>,
    pub steps: u64,
}

impl Default for Machine {
    fn default() -> Self {
        Self::new()
    }
}

impl Machine {
    pub fn new() -> Machine {
        Machine { mem: vec![0; CELLS], input: VecDeque::new(), output: Vec::new(), steps: 0 }
    }

    fn load(&mut self, a: usize) -> Result<u64, Fault> {
        if a == IO {
            self.input.pop_front().ok_or(Fault::InputExhausted)
        } else {
            Ok(self.mem[a])
        }
    }

    fn store(&mut self, a: usize, v: u64) {
        match a {
            PI => self.mem[PI] = v % 10_000,
            IO => self.output.push(v),
            _ => self.mem[a] = v,
        }
    }

    /// A simple load, or an iterated one (1.62) when the ε digit is 1.
    fn operand(&mut self, k: u64, at: usize, word: u64) -> Result<u64, Fault> {
        let c = (k % 1000) as usize;
        match k / 1000 {
            0 => self.load(c),
            1 => {
                let a = (self.load(c)? % 1000) as usize;
                self.load(a)
            }
            _ => Err(Fault::BadInstruction { at, word }),
        }
    }

    /// One turn of the fundamental cyclic program (1.21, 1.22, 1.23).
    /// Returns false when the machine stops.
    pub fn step(&mut self) -> Result<bool, Fault> {
        let pc = self.mem[PI];
        if pc >= CELLS as u64 {
            return Err(Fault::PiOutOfRange(pc));
        }
        let at = pc as usize;
        let word = self.load(at)?;
        self.mem[PI] = (pc + 1) % 10_000;
        self.steps += 1;
        if word == 0 {
            // The stop instruction. π = 0 lands here too, because address 000 is π itself.
            return Ok(false);
        }
        let (k1, cop, k2, kr) = (word / M1, word / M2 % 100, word / M3 % 10_000, word % 10_000);
        let v = if cop == TRANSFER {
            self.operand(k2, at, word)?
        } else {
            let op = Op::from_code(cop).ok_or(Fault::BadInstruction { at, word })?;
            let a = self.operand(k1, at, word)?;
            let b = self.operand(k2, at, word)?;
            op.apply(a, b)?
        };
        let cr = (kr % 1000) as usize;
        match kr / 1000 {
            0 => self.store(cr, v),
            1 => {
                let a = (self.load(cr)? % 1000) as usize;
                self.store(a, v)
            }
            _ => return Err(Fault::BadInstruction { at, word }),
        }
        Ok(true)
    }

    pub fn run(&mut self, limit: u64) -> Result<(), Fault> {
        let end = self.steps + limit;
        while self.step()? {
            if self.steps >= end {
                return Err(Fault::StepLimit);
            }
        }
        Ok(())
    }
}
