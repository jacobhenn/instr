use crate::parse::{LabelError, Token};

use ariadne::{Color, Fmt, Label, Report, ReportKind, Source};

use chumsky::prelude::Simple;

fn say_token(chr: &char) -> String {
    match chr {
        '\n' => "newline".to_string(),
        ' ' => "space".to_string(),
        ',' => "comma".to_string(),
        '.' => "dot".to_string(),
        '\'' => "single quote".to_string(),
        '"' => "double quote".to_string(),
        '`' => "backtick".to_string(),
        other => format!("'{}'",  other.escape_default()),
    }
}

pub fn emit_parse_error(src: &str, err: Simple<char>, path: &str) {
    let msg = if let chumsky::error::SimpleReason::Custom(msg) = err.reason() {
        msg.clone()
    } else {
        format!(
            "Expected {}{}, found {}",
            if err.expected().len() == 0 {
                "something else".to_string()
            } else {
                let mut expecteds = err
                    .expected()
                    .map(|expected| match expected {
                        Some(expected) => say_token(expected),
                        None => "end of input".to_string(),
                    })
                    .collect::<Vec<_>>();
                if expecteds.len() == 1 {
                    expecteds.pop().unwrap()
                } else if expecteds.len() == 2 {
                    format!("{} or {}", expecteds[0], expecteds[1])
                } else {
                    let last = expecteds.pop().unwrap();
                    format!("{}, or {last}", expecteds.join(", "))
                }
            },
            if let Some(label) = err.label() {
                format!(" while parsing {}", label)
            } else {
                String::new()
            },
            err.found()
                .map(|c| format!(r"{}", say_token(c).fg(Color::Red)))
                .unwrap_or_else(|| "end of input".to_string()),
        )
    };

    let report = Report::build(ReportKind::Error, path, err.span().start)
        .with_code(0)
        .with_message(msg)
        .with_label(
            Label::new((path, err.span()))
                .with_message(match err.reason() {
                    chumsky::error::SimpleReason::Custom(msg) => msg,
                    _ => "unexpected token",
                })
                .with_color(Color::Red),
        );

    let report = match err.reason() {
        chumsky::error::SimpleReason::Unclosed { span, delimiter } => report.with_label(
            Label::new((path, span.clone()))
                .with_message(format!(
                    "Unclosed delimiter {}",
                    delimiter.fg(Color::Yellow)
                ))
                .with_color(Color::Yellow),
        ),
        chumsky::error::SimpleReason::Unexpected => report,
        chumsky::error::SimpleReason::Custom(_) => report,
    };

    report.finish().print((path, Source::from(&src))).unwrap();
    println!();
}

pub fn emit_label_error(err: LabelError, src: &str, path: &str) {
    let report = match err {
        LabelError::Unknown(Token { inner, span }) => {
            Report::build(ReportKind::Error, path, span.start)
                .with_code(1)
                .with_message(format!("unknown label `{inner}`"))
                .with_label(
                    Label::new((path, span))
                        .with_message("this label is not defined")
                        .with_color(Color::Red),
                )
        }
        LabelError::Redefined {
            label,
            first,
            second,
        } => Report::build(ReportKind::Error, path, second.start)
            .with_code(2)
            .with_message(format!("multiple definitions of label `{label}`"))
            .with_label(
                Label::new((path, first))
                    .with_message("first defined here")
                    .with_color(Color::Blue),
            )
            .with_label(
                Label::new((path, second))
                    .with_message("later defined here")
                    .with_color(Color::Red),
            ),
    };

    report.finish().print((path, Source::from(&src))).unwrap();
    println!();
}
