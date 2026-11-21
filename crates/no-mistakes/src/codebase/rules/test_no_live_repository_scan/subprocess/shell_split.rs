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
    /// The name and `=` were unquoted, so this word is an assignment prefix.
    assignment: bool,
}

/// Words of a static shell string. A quote wraps one argument, including the
/// value of `NAME="..."`. An unclosed quote or a backslash is not static.
/// `$` and a backtick are literal inside single quotes. Inside double quotes,
/// or in unquoted text of a command that also contains a quote, they are not.
fn shell_words(text: &str) -> Option<Vec<Word>> {
    if text.contains(['"', '\'']) {
        quoted_words(text)
    } else {
        Some(
            text.split_whitespace()
                .map(|part| Word {
                    text: part.to_string(),
                    quoted: false,
                    assignment: part
                        .split_once('=')
                        .is_some_and(|(name, _)| identifier(name)),
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
            assignment: false,
        };
        let mut name_open = true;
        while let Some(character) = chars.peek().copied() {
            if character.is_whitespace() {
                break;
            }
            if character == '\\' {
                return None;
            }
            if character == '\'' || character == '"' {
                if name_open {
                    name_open = false;
                }
                chars.next();
                word.quoted = true;
                loop {
                    let inner = chars.next()?;
                    if inner == character {
                        break;
                    }
                    // Single quotes keep `$` and backticks. Double quotes do not.
                    if inner == '\\' || inner == '\n' || inner == '\r' {
                        return None;
                    }
                    if character == '"' && matches!(inner, '`' | '$') {
                        return None;
                    }
                    word.text.push(inner);
                }
            } else {
                if matches!(character, '`' | '$') {
                    return None;
                }
                chars.next();
                if name_open && character == '=' {
                    word.assignment = identifier(&word.text);
                    name_open = false;
                }
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
    while words.first().is_some_and(|word| word.assignment) || cd_and(words) {
        if words.first().is_some_and(|word| word.assignment) {
            words.remove(0);
        } else {
            directory = words.get(1).map(|word| word.text.clone());
            words.drain(..3);
        }
    }
    directory
}

fn identifier(text: &str) -> bool {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) if first.is_ascii_alphabetic() || first == '_' => {
            chars.all(|character| character.is_ascii_alphanumeric() || character == '_')
        }
        _ => false,
    }
}

fn cd_and(words: &[Word]) -> bool {
    let (Some(cd), Some(dir), Some(and)) = (words.first(), words.get(1), words.get(2)) else {
        return false;
    };
    // `cd` and `&&` are operators only when they are unquoted tokens.
    !cd.quoted
        && !and.quoted
        && cd.text == "cd"
        && and.text == "&&"
        && (dir.quoted || dir.text != "&&")
        && !dir.text.starts_with('-')
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
