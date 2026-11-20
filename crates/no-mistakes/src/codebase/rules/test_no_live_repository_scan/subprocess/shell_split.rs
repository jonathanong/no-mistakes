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
    if text.contains(['"', '\'']) {
        return unmatched(values);
    }
    let mut parts: Vec<Token> = text.split_whitespace().map(known).collect();
    let shell_dir = strip_prefixes(&mut parts);
    let mut parts = parts.into_iter();
    Invocation {
        executable: parts.next().unwrap_or(Token::Unknown),
        argv: parts.collect(),
        options: options(values),
        shell_dir,
    }
}

fn unmatched(values: &[ArgumentValue]) -> Invocation {
    Invocation {
        executable: Token::Unknown,
        argv: Vec::new(),
        options: options(values),
        shell_dir: None,
    }
}

fn strip_prefixes(tokens: &mut Vec<Token>) -> Option<String> {
    let mut directory = None;
    while assignment(tokens.first()) || cd_and(tokens) {
        if assignment(tokens.first()) {
            tokens.remove(0);
        } else {
            directory = match tokens.get(1) {
                Some(Token::Known(dir)) => Some(dir.clone()),
                _ => None,
            };
            tokens.drain(..3);
        }
    }
    directory
}

fn assignment(token: Option<&Token>) -> bool {
    let Some(Token::Known(text)) = token else {
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

fn cd_and(tokens: &[Token]) -> bool {
    let (Some(Token::Known(cd)), Some(Token::Known(dir)), Some(Token::Known(and))) =
        (tokens.first(), tokens.get(1), tokens.get(2))
    else {
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
