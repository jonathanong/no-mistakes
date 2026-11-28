use super::{
    super::lex::{identifier, Word},
    frames::Frame,
};
use crate::fx::{FxHashMap, FxHashSet};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Leave {
    None,
    Flow,
    Exit,
}

impl Leave {
    pub fn is_leaving(self) -> bool {
        self != Self::None
    }

    pub fn combine(self, other: Self) -> Self {
        match (self, other) {
            (Self::Exit, _) | (_, Self::Exit) => Self::Exit,
            (Self::Flow, _) | (_, Self::Flow) => Self::Flow,
            (Self::None, Self::None) => Self::None,
        }
    }
}

#[derive(Clone, Copy)]
pub(super) struct Guard<'a> {
    pub name: &'a str,
    pub when_empty: bool,
}

impl<'a> Guard<'a> {
    pub(super) fn parse(words: &[&Word<'a>]) -> Option<Self> {
        let (negated, words) = match words {
            [first, rest @ ..] if first.raw == "!" => (true, rest),
            _ => (false, words),
        };
        let guard = match words {
            [open, length, op, value, close]
                if matches!((open.raw, close.raw), ("[[", "]]") | ("[", "]")) =>
            {
                comparison(length.raw, op.raw, value.raw)?
            }
            [arithmetic] => parse_arithmetic(arithmetic.raw)?,
            _ => return None,
        };
        Some(Self {
            when_empty: guard.when_empty != negated,
            ..guard
        })
    }
}

fn parse_arithmetic(raw: &str) -> Option<Guard<'_>> {
    let expr = raw.strip_prefix("((")?.strip_suffix("))")?.trim();
    if let Some(length) = expr.strip_prefix('!') {
        return Some(Guard {
            name: length_name(length.trim())?,
            when_empty: true,
        });
    }
    if let Some(offset) = expr.find(['=', '!', '<', '>']) {
        let (length, rhs) = expr.split_at(offset);
        let end = rhs.find(|ch: char| !matches!(ch, '=' | '!' | '<' | '>'))?;
        let (op, value) = rhs.split_at(end);
        comparison(length.trim(), op, value.trim())
    } else {
        Some(Guard {
            name: length_name(expr)?,
            when_empty: false,
        })
    }
}

fn comparison<'a>(length: &'a str, op: &str, value: &str) -> Option<Guard<'a>> {
    let when_empty = match (op, value) {
        ("-eq" | "==", "0") | ("-lt" | "<", "1") | ("-le" | "<=", "0") => true,
        ("-ne" | "!=" | "-gt" | ">", "0") | ("-ge" | ">=", "1") => false,
        _ => return None,
    };
    Some(Guard {
        name: length_name(length)?,
        when_empty,
    })
}

fn length_name(word: &str) -> Option<&str> {
    let unquoted = word
        .strip_prefix('"')
        .and_then(|body| body.strip_suffix('"'))
        .unwrap_or(word);
    let name = unquoted
        .strip_prefix("${#")?
        .strip_suffix("[@]}")
        .or_else(|| unquoted.strip_prefix("${#")?.strip_suffix("[*]}"))?;
    identifier(name).then_some(name)
}

pub(super) fn loop_levels(words: &[&Word<'_>]) -> usize {
    words.get(1).map(|arg| jump_count(arg.raw)).unwrap_or(1)
}

fn strip_static_quotes(raw: &str) -> &str {
    let bytes = raw.as_bytes();
    if bytes.len() >= 2
        && (bytes[0] == b'\'' || bytes[0] == b'"')
        && bytes[0] == bytes[bytes.len() - 1]
    {
        &raw[1..raw.len() - 1]
    } else {
        raw
    }
}

fn jump_count(raw: &str) -> usize {
    strip_static_quotes(raw).parse::<usize>().unwrap_or(1)
}

pub(super) fn leaves(
    words: &[&Word<'_>],
    scope: usize,
    frames: &[Frame<'_>],
    fn_exits: &FxHashSet<&str>,
    seen_functions: &FxHashMap<&str, usize>,
) -> Leave {
    if let Some(first) = words.first() {
        if fn_exits.contains(first.raw) {
            return Leave::Exit;
        }
        if seen_functions.contains_key(first.raw) {
            return Leave::None;
        }
    }
    let (name, argument) = match words {
        [name] => (name.raw, None),
        [name, arg] => (name.raw, Some(strip_static_quotes(arg.raw))),
        _ => return Leave::None,
    };
    if words.len() == 2
        && !matches!(name, "break" | "continue")
        && argument.is_some_and(|raw| raw.parse::<i64>().is_err())
    {
        return Leave::None;
    }
    match name {
        "exit" => Leave::Exit,
        "return" if scope > 0 => Leave::Flow,
        "continue" | "break" => {
            let requested = argument.map(jump_count).unwrap_or(1);
            if requested == 0 {
                return Leave::None;
            }
            let available = frames
                .iter()
                .rev()
                .take_while(|frame| !matches!(frame, Frame::Group(group) if group.isolated))
                .filter(|frame| matches!(frame, Frame::Repeat(_)))
                .count();
            if available == 0 {
                Leave::None
            } else {
                Leave::Flow
            }
        }
        _ => Leave::None,
    }
}

pub(super) fn nounset<'w, 'a: 'w>(mut words: impl Iterator<Item = &'w Word<'a>>) -> Option<bool> {
    let mut changed = None;
    while let Some(word) = words.next() {
        let Some((enable, flags)) = flag(word.raw) else {
            break;
        };
        if flags.contains('u') {
            changed = Some(enable);
        }
        if flags.contains('o') && words.next().is_some_and(|option| option.raw == "nounset") {
            changed = Some(enable);
        }
    }
    changed
}

pub(super) fn flag(word: &str) -> Option<(bool, &str)> {
    let (enable, options) = word
        .strip_prefix('-')
        .map(|options| (true, options))
        .or_else(|| word.strip_prefix('+').map(|options| (false, options)))?;
    (!options.is_empty() && options.bytes().all(|byte| byte.is_ascii_alphabetic()))
        .then_some((enable, options))
}
