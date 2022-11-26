use crate::{args::Args, run::State};

use std::fs;

use anyhow::{Context, Error};

use chumsky::Parser;

mod instrs;

mod parse;

mod run;

mod args;

mod err;

#[cfg(test)]
mod tests;

fn main() -> Result<(), Error> {
    env_logger::init();

    let args: Args = argh::from_env();
    let src = fs::read_to_string(&args.path).context("failed to read input file")?;

    let (ast, errs) = parse::root().parse_recovery(src.as_str());

    let path = args.path.to_string_lossy();

    if !errs.is_empty() {
        for err in errs {
            err::emit_parse_error(&src, err, &path);
        }
    }

    if let Some(ast) = ast {
        let (program, table) = match parse::resolve(ast) {
            Ok(x) => x,
            Err(errs) => {
                for err in errs {
                    err::emit_label_error(err, &src, &path);
                }

                return Ok(());
            }
        };
        let mut state = State::new(&program, table);
        state.run()?;
    }

    Ok(())
}
