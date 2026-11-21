use super::{Invocation, OptionsSlot, Token};
use crate::codebase::ts_source::module_facts::arguments::ArgumentValue;
use serde_json::Value;

pub(super) fn flag_true(values: &[ArgumentValue]) -> bool {
    values.iter().skip(1).any(|value| match value {
        ArgumentValue::Object(fields) => matches!(
            fields.get("shell"),
            Some(ArgumentValue::Scalar(Value::Bool(true)))
        ),
        _ => false,
    })
}

pub(super) fn split(values: &[ArgumentValue]) -> Invocation {
    let Some(ArgumentValue::Scalar(Value::String(text))) = values.first() else {
        return unmatched(values);
    };
    let Some(mut words) = shell_words(text) else {
        return unmatched(values);
    };
    let shell_dir = strip_prefixes(&mut words);
    // A quoted executable is not a static command name. A quoted argument is.
    if words.first().is_some_and(|word| word.quoted) {
        return unmatched(values);
    }
    let mut parts = words.into_iter().map(|word| known(&word.text));
    Invocation {
        executable: parts.next().unwrap_or(Token::Unknown),
        argv: parts.collect(),
        options: options(values),
        shell_dir,
    }
}

struct Word {
    text: String,
    quoted: bool,
}

/// Words of a static shell string. A quote wraps one argument, including the
/// value of `NAME="..."`. An unclosed quote, a backslash, or `$` or a backtick
/// inside quotes is not static.
fn shell_words(text: &str) -> Option<Vec<Word>> {
    if text.contains(['"', '\'']) {
        quoted_words(text)
    } else {
        Some(
            text.split_whitespace()
                .map(|part| Word {
                    text: part.to_string(),
                    quoted: false,
                })
                .collect(),
        )
    }
}

fn quoted_words(text: &str) -> Option<Vec<Word>> {
    let mut words = Vec::new();
    let mut chars = text.chars().peekable();
    loop {
        while chars
            .peek()
            .is_some_and(|character| character.is_whitespace())
        {
            chars.next();
        }
        if chars.peek().is_none() {
            break;
        }
        let mut word = Word {
            text: String::new(),
            quoted: false,
        };
        while let Some(character) = chars.peek().copied() {
            if character.is_whitespace() {
                break;
            }
            if character == '\\' {
                return None;
            }
            if character == '\'' || character == '"' {
                chars.next();
                word.quoted = true;
                loop {
                    let inner = chars.next()?;
                    if inner == character {
                        break;
                    }
                    if matches!(inner, '\\' | '`' | '$' | '\n' | '\r') {
                        return None;
                    }
                    word.text.push(inner);
                }
            } else {
                chars.next();
                word.text.push(character);
            }
        }
        words.push(word);
    }
    Some(words)
}

fn unmatched(values: &[ArgumentValue]) -> Invocation {
    Invocation {
        executable: Token::Unknown,
        argv: Vec::new(),
        options: options(values),
        shell_dir: None,
    }
}

fn strip_prefixes(words: &mut Vec<Word>) -> Option<String> {
    let mut directory = None;
    while assignment(words.first().map(|word| word.text.as_str())) || cd_and(words) {
        if assignment(words.first().map(|word| word.text.as_str())) {
            words.remove(0);
        } else {
            directory = words.get(1).map(|word| word.text.clone());
            words.drain(..3);
        }
    }
    directory
}

fn assignment(text: Option<&str>) -> bool {
    let Some(text) = text else {
        return false;
    };
    let Some((name, _)) = text.split_once('=') else {
        return false;
    };
    let mut chars = name.chars();
    match chars.next() {
        Some(first) if first.is_ascii_alphabetic() || first == '_' => {
            chars.all(|character| character.is_ascii_alphanumeric() || character == '_')
        }
        _ => false,
    }
}

fn cd_and(words: &[Word]) -> bool {
    let (Some(cd), Some(dir), Some(and)) = (
        words.first().map(|word| word.text.as_str()),
        words.get(1).map(|word| word.text.as_str()),
        words.get(2).map(|word| word.text.as_str()),
    ) else {
        return false;
    };
    cd == "cd" && and == "&&" && dir != "&&" && !dir.starts_with('-')
}

fn known(part: &str) -> Token {
    Token::Known(part.to_string())
}

fn options(values: &[ArgumentValue]) -> OptionsSlot {
    match values.get(1) {
        None => OptionsSlot::Absent,
        Some(ArgumentValue::Object(_)) => OptionsSlot::Object(1),
        Some(_) => OptionsSlot::Unknown,
    }
}
