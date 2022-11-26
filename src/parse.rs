use crate::instrs::{Instr, Reg, Val};

use std::{collections::HashMap, ops::Range};

use chumsky::prelude::*;
use log::debug;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Token {
    pub inner: String,
    pub span: Range<usize>,
}

type Ast = Vec<AstLine>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AstLine {
    Instr(Instr),
    Label(Token),
    Error,
}

fn num() -> impl Parser<char, i64, Error = Simple<char>> {
    just("-")
        .or_not()
        .then(text::int(10))
        .try_map(|(minus, s): (_, String), r| {
            s.parse::<i64>()
                .map_err(|e| Simple::custom(r, format!("failed to parse integer: {e}")))
                .map(|n| if minus.is_none() { n } else { -n })
        })
        .labelled("number")
}

fn val() -> impl Parser<char, Val, Error = Simple<char>> {
    num()
        .map(Val::Num)
        .or(just("x").to(Val::X))
        .labelled("number or x")
}

pub fn root() -> impl Parser<char, Ast, Error = Simple<char>> {
    let space = || {
        one_of("\t ")
            .repeated()
            .at_least(1)
            .labelled("horizontal whitespace")
    };
    let space0 = one_of("\t ").repeated().labelled("horizontal whitespace");

    let label = text::ident()
        .map_with_span(|inner, span| Token { inner, span })
        .labelled("label");

    let reg = choice((just("x").to(Reg::X), just("acc").to(Reg::Acc))).labelled("register");

    choice((
        just("gol").to(Instr::Gol),
        just("gor").to(Instr::Gor),
        just("get").then(space()).ignore_then(reg).map(Instr::Get),
        just("put").then(space()).ignore_then(reg).map(Instr::Put),
        just("jmp").then(space()).ignore_then(label).map(Instr::Jmp),
        just("jnz")
            .ignore_then(space())
            .ignore_then(reg)
            .then_ignore(space())
            .then(label)
            .map(|(r, l)| Instr::Jnz(r, l)),
        just("jlz")
            .ignore_then(space())
            .ignore_then(reg)
            .then_ignore(space())
            .then(label)
            .map(|(r, l)| Instr::Jlz(r, l)),
        just("sav").to(Instr::Sav),
        just("ret").to(Instr::Ret),
        just("inp").to(Instr::Inp),
        just("out").to(Instr::Out),
        just("set").then(space()).ignore_then(val()).map(Instr::Set),
        just("add").then(space()).ignore_then(val()).map(Instr::Add),
        just("mul").then(space()).ignore_then(val()).map(Instr::Mul),
        just("div").then(space()).ignore_then(val()).map(Instr::Div),
        just("dec").to(Instr::Dec),
    ))
    .map(AstLine::Instr)
    .labelled("instruction")
    .or(text::ident()
        .then_ignore(just(":"))
        .map_with_span(|inner, span| AstLine::Label(Token { inner, span }))
        .labelled("label"))
    .recover_with(skip_until(['\n'], |_| AstLine::Error))
    .then_ignore(
        space0
            .then(text::newline())
            .labelled("trailing newline")
            .then(text::whitespace()),
    )
    .repeated()
    .then_ignore(end())
}

pub enum LabelError {
    Unknown(Token),
    Redefined {
        label: String,
        first: Range<usize>,
        second: Range<usize>,
    },
}

pub fn resolve(ast: Ast) -> Result<(Vec<Instr>, HashMap<String, usize>), Vec<LabelError>> {
    let mut labels = HashMap::<String, usize>::new();
    let mut spans = HashMap::<&str, Range<usize>>::new();
    let mut errs = Vec::new();

    let mut instr_idx = 0;
    for line in &ast {
        if let AstLine::Label(token) = line {
            if let Some(first) = spans.get(token.inner.as_str()) {
                errs.push(LabelError::Redefined {
                    label: token.inner.clone(),
                    first: first.clone(),
                    second: token.span.clone(),
                });
            } else {
                labels.insert(token.inner.clone(), instr_idx);
                spans.insert(&token.inner, token.span.clone());
                debug!("found label {} @ {}", token.inner, instr_idx);
            }
        } else {
            instr_idx += 1;
        }
    }

    for line in &ast {
        if let AstLine::Instr(Instr::Jmp(token) | Instr::Jnz(_, token) | Instr::Jlz(_, token)) =
            line
        {
            if !spans.contains_key(token.inner.as_str()) {
                errs.push(LabelError::Unknown(token.clone()))
            }
        }
    }

    let program = ast
        .into_iter()
        .filter_map(|line| match line {
            AstLine::Instr(i) => Some(i),
            _ => None,
        })
        .collect();

    if errs.is_empty() {
        Ok((program, labels))
    } else {
        Err(errs)
    }
}
