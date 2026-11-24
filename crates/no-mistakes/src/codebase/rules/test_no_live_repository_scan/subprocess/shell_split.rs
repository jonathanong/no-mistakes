use super::{Invocation, OptionsSlot, Token};
use crate::codebase::ts_source::module_facts::arguments::ArgumentValue;
use serde_json::Value;
use std::collections::BTreeSet;
use std::mem::take;

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
    let mut success = BTreeSet::from([None]);
    let mut failure = BTreeSet::new();
    let mut previous = lex::Sep::And;
    let options = options(values);
    let mut out = Vec::new();
    for part in parts {
        if part.words.is_empty() {
            continue;
        }
        let executable_index = part
            .words
            .iter()
            .position(|word| !word.assignment)
            .unwrap_or(part.words.len());
        // Skipped commands retain the prior list's status and directory.
        let directories = match previous {
            lex::Sep::And => take(&mut success),
            lex::Sep::Or => take(&mut failure),
            lex::Sep::Semi => {
                success.append(&mut failure);
                take(&mut success)
            }
        };
        let dir = part
            .next
            .as_ref()
            .and_then(|_| cd_dir(&part.words[executable_index..]));
        previous = part.next.unwrap_or(lex::Sep::And);
        if let Some(dir) = dir {
            success.insert(Some(dir.to_owned()));
            if !matches!(previous, lex::Sep::Semi) {
                failure.extend(directories);
            }
            continue;
        }
        let mut tokens = part
            .words
            .into_iter()
            .skip(executable_index)
            .map(|word| Token::Known(word.text));
        if let Some(executable) = tokens.next() {
            out.push(Invocation {
                executable,
                argv: tokens.collect(),
                options,
                shell_dirs: Some(directories.iter().cloned().collect()),
            });
        }
        success.extend(directories.iter().cloned());
        failure.extend(directories);
    }
    out
}

fn cd_dir(words: &[lex::Word]) -> Option<&str> {
    let [cd, dir] = words else {
        return None;
    };
    (cd.text == "cd" && !dir.text.starts_with('-')).then_some(dir.text.as_str())
}

fn unmatched(values: &[ArgumentValue]) -> Invocation {
    Invocation {
        executable: Token::Unknown,
        argv: Vec::new(),
        options: options(values),
        shell_dirs: None,
    }
}

fn options(values: &[ArgumentValue]) -> OptionsSlot {
    match values.get(1) {
        None => OptionsSlot::Absent,
        Some(ArgumentValue::Object(_)) => OptionsSlot::Object(1),
        Some(_) => OptionsSlot::Unknown,
    }
}

#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
#[path = "tests/shell_split_tests.rs"]
mod tests;
