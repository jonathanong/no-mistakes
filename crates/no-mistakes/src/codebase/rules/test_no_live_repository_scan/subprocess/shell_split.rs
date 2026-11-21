use super::{Invocation, OptionsSlot, Token};
use crate::codebase::ts_source::module_facts::arguments::ArgumentValue;
use serde_json::Value;

#[path = "shell_lex.rs"]
mod lex;

pub(super) fn flag_true(values: &[ArgumentValue]) -> bool {
    values.iter().skip(1).any(|value| match value {
        ArgumentValue::Object(fields) => matches!(
            fields.get("shell"),
            Some(ArgumentValue::Scalar(Value::Bool(true)))
        ),
        _ => false,
    })
}

pub(super) fn splits(values: &[ArgumentValue]) -> Vec<Invocation> {
    let Some(ArgumentValue::Scalar(Value::String(text))) = values.first() else {
        return vec![unmatched(values)];
    };
    let Some(parts) = lex::segments(text) else {
        return vec![unmatched(values)];
    };
    let commands = commands(parts, values);
    if commands.is_empty() {
        vec![unmatched(values)]
    } else {
        commands
    }
}

fn commands(parts: Vec<lex::Part>, values: &[ArgumentValue]) -> Vec<Invocation> {
    let mut directory = None;
    let mut out = Vec::new();
    for part in parts {
        let mut words = part.words;
        while words.first().is_some_and(|word| word.assignment) {
            words.remove(0);
        }
        if let Some(dir) = cd_dir(&words, part.next) {
            directory = Some(dir);
            continue;
        }
        let mut tokens = words.into_iter().map(|word| Token::Known(word.text));
        let Some(executable) = tokens.next() else {
            continue;
        };
        out.push(Invocation {
            executable,
            argv: tokens.collect(),
            options: options(values),
            shell_dir: directory.clone(),
        });
    }
    out
}

/// `cd <dir>` before `&&` or `;` sets the directory. `||` leaves it unchanged.
fn cd_dir(words: &[lex::Word], next: Option<lex::Sep>) -> Option<String> {
    if !matches!(next, Some(lex::Sep::And | lex::Sep::Semi)) {
        return None;
    }
    let [cd, dir] = words else {
        return None;
    };
    (cd.text == "cd" && !dir.text.starts_with('-')).then(|| dir.text.clone())
}

fn unmatched(values: &[ArgumentValue]) -> Invocation {
    Invocation {
        executable: Token::Unknown,
        argv: Vec::new(),
        options: options(values),
        shell_dir: None,
    }
}

fn options(values: &[ArgumentValue]) -> OptionsSlot {
    match values.get(1) {
        None => OptionsSlot::Absent,
        Some(ArgumentValue::Object(_)) => OptionsSlot::Object(1),
        Some(_) => OptionsSlot::Unknown,
    }
}
