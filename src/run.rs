use crate::{
    instrs::{Instr, Reg, Val},
    parse::Token,
};

use std::{collections::HashMap, io, num::TryFromIntError};

use anyhow::{Context, Error};
use log::trace;

pub struct State<'a> {
    x: i64,
    acc: i64,
    tape: Vec<i64>,
    cursor: usize,
    program: &'a [Instr],
    labels: HashMap<String, usize>,
    instr_ptr: usize,
    jumped: bool,
    input_buf: String,
    input_cursor: usize,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum ControlFlow {
    Continue,
    Exit,
}

impl<'a> State<'a> {
    pub fn new(program: &'a [Instr], labels: HashMap<String, usize>) -> Self {
        Self {
            x: 0,
            acc: 0,
            tape: Vec::new(),
            cursor: 0,
            program,
            labels,
            instr_ptr: 0,
            jumped: false,
            input_buf: String::new(),
            input_cursor: 0,
        }
    }

    fn reg(&self, reg: Reg) -> i64 {
        match reg {
            Reg::X => self.x,
            Reg::Acc => self.acc,
        }
    }

    fn reg_mut(&mut self, reg: Reg) -> &mut i64 {
        match reg {
            Reg::X => &mut self.x,
            Reg::Acc => &mut self.acc,
        }
    }

    fn grow(&mut self) {
        if self.cursor >= self.tape.len() {
            self.tape.resize(self.cursor + 1, 0);
        }
    }

    fn cur(&mut self) -> i64 {
        self.grow();
        self.tape[self.cursor]
    }

    fn cur_mut(&mut self) -> &mut i64 {
        self.grow();
        &mut self.tape[self.cursor]
    }

    fn val_of(&self, val: Val) -> i64 {
        match val {
            Val::Num(n) => n,
            Val::X => self.x,
        }
    }

    fn run_instr(&mut self) -> Result<ControlFlow, Error> {
        let Some(instr) = self.program.get(self.instr_ptr) else {
            return Ok(ControlFlow::Exit);
        };

        trace!("x: {}, acc: {}", self.x, self.acc);
        trace!("tape: {:?}", self.tape);
        trace!("{instr:?}");

        match instr {
            Instr::Gol => self.gol(),
            Instr::Gor => self.gor(),
            Instr::Get(reg) => self.get(*reg),
            Instr::Put(reg) => self.put(*reg),
            Instr::Jmp(label) => self.jmp(label),
            Instr::Jnz(reg, label) => self.jnz(*reg, label),
            Instr::Jlz(reg, label) => self.jlz(*reg, label),
            Instr::Sav => self.sav(),
            Instr::Ret => self.ret().context("`ret` instruction failed")?,
            Instr::Inp => self.inp()?,
            Instr::Out => self.out(),
            Instr::Set(val) => self.set(*val),
            Instr::Add(val) => self.add(*val),
            Instr::Mul(val) => self.mul(*val),
            Instr::Div(val) => self.div(*val),
            Instr::Dec => self.dec(),
        }

        if self.jumped {
            self.jumped = false;
        } else {
            self.instr_ptr += 1;
        }

        Ok(ControlFlow::Continue)
    }

    pub fn run(&mut self) -> Result<(), Error> {
        while self.run_instr()? == ControlFlow::Continue {}
        Ok(())
    }
}

impl<'a> State<'a> {
    fn gol(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }

    fn gor(&mut self) {
        self.cursor += 1;
    }

    fn get(&mut self, reg: Reg) {
        *self.cur_mut() = self.reg(reg);
    }

    fn put(&mut self, reg: Reg) {
        *self.reg_mut(reg) = self.cur();
    }

    fn jmp(&mut self, label: &Token) {
        self.instr_ptr = self.labels[&label.inner];
        self.jumped = true;
        trace!("    jumped");
    }

    fn jnz(&mut self, reg: Reg, label: &Token) {
        if self.reg(reg) != 0 {
            self.jmp(label);
        } else {
            trace!("    did not jump")
        }
    }

    fn jlz(&mut self, reg: Reg, label: &Token) {
        if self.reg(reg) < 0 {
            self.jmp(label);
        } else {
            trace!("    did not jump");
        }
    }

    fn sav(&mut self) {
        *self.cur_mut() = self.instr_ptr as i64;
    }

    fn ret(&mut self) -> Result<(), TryFromIntError> {
        let label = usize::try_from(self.cur())?;
        self.instr_ptr = label;
        Ok(())
    }

    fn inp(&mut self) -> Result<(), Error> {
        if self.input_cursor == self.input_buf.len() {
            self.input_buf.clear();
            io::stdin()
                .read_line(&mut self.input_buf)
                .context("couldn't read line of stdin")?;
            if self.input_cursor == 0 {
                return self.inp();
            } else {
                *self.cur_mut() = '\n' as i64;
            }
            self.input_cursor = 0;
            return Ok(());
        }

        *self.cur_mut() = self.input_buf.as_bytes()[self.input_cursor] as i64;
        self.input_cursor += 1;

        trace!("got input: {}", self.cur());

        Ok(())
    }

    fn out(&mut self) {
        let Ok(n) = u32::try_from(self.cur()) else { return; };
        let Ok(chr) = char::try_from(n) else { return; };
        print!("({}: {chr})", self.cur());
    }

    fn set(&mut self, val: Val) {
        *self.cur_mut() = self.val_of(val);
    }

    fn add(&mut self, val: Val) {
        *self.cur_mut() += self.val_of(val);
    }

    fn mul(&mut self, val: Val) {
        *self.cur_mut() *= self.val_of(val);
    }

    fn div(&mut self, val: Val) {
        *self.cur_mut() /= self.val_of(val);
    }

    fn dec(&mut self) {
        self.acc -= 1;
    }
}
