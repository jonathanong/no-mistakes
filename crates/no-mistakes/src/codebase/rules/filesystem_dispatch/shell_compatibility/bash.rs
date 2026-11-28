use super::lex::{assignment, command_words, identifier, Lexer, Shell, Word};
use super::read_targets::targets;
use crate::codebase::command_lex::words::{split_quoted_words, Whitespace};
use crate::fx::FxHashMap;

pub const RULE_ID: &str = "bash-empty-array-nounset";

pub(super) fn scan(source: &str) -> Vec<(&str, usize)> {
    let mut arrays = FxHashMap::default();
    let mut active = false;
    let mut hits = Vec::new();
    Lexer::new(source, Shell::Bash).commands(|words, _| {
        if active {
            hits.extend(
                words
                    .iter()
                    .flat_map(|word| word.refs.iter().copied())
                    .filter(|(name, _)| arrays.get(*name) == Some(&false)),
            );
        }
        if let Some(read) = targets(words) {
            for name in read.names {
                if read.keep {
                    arrays.entry(name).or_insert(false);
                } else {
                    arrays.insert(name, false);
                }
            }
            return;
        }
        let mut command = command_words(words).peekable();
        let Some(first) = command.peek() else {
            return;
        };
        match first.raw {
            "set" => active = nounset(command.skip(1), active),
            "unset" => forget_unset(command.skip(1), &mut arrays),
            "declare" | "local" => {
                command.next();
                let mut array = false;
                while let Some((enable, flags)) = command.peek().and_then(|word| flag(word.raw)) {
                    if flags.contains('a') {
                        array = enable;
                    }
                    command.next();
                }
                command.next_if(|word| word.raw == "--");
                for word in command {
                    if array && identifier(word.raw) {
                        arrays.entry(word.raw.to_string()).or_insert(false);
                    }
                    if let Some(assignment) = array_assignment(word.raw) {
                        remember_array(&mut arrays, word, assignment, array);
                    }
                }
            }
            _ => {
                let assignments: Option<Vec<_>> = command
                    .map(|word| array_assignment(word.raw).map(|assignment| (word, assignment)))
                    .collect();
                for (word, assignment) in assignments.into_iter().flatten() {
                    remember_array(&mut arrays, word, assignment, false);
                }
            }
        }
    });
    hits
}

fn remember_array<'a>(
    arrays: &mut FxHashMap<String, bool>,
    word: &Word<'a>,
    (name, append): (&'a str, bool),
    declared: bool,
) {
    if let Some(literal) = word.array_literal {
        let nonempty = arrays.entry(name.to_string()).or_default();
        *nonempty = literal || (append && *nonempty);
    } else if declared {
        arrays.insert(name.to_string(), true);
    } else if let Some(nonempty) = arrays.get_mut(name) {
        *nonempty = true;
    }
}

fn array_assignment(word: &str) -> Option<(&str, bool)> {
    if let Some(assignment) = assignment(word) {
        return Some(assignment);
    }
    let (lhs, _) = word.split_once('=')?;
    let append = lhs.ends_with('+');
    let lhs = lhs.strip_suffix('+').unwrap_or(lhs);
    indexed_name(lhs).map(|name| (name, append))
}

fn indexed_name(word: &str) -> Option<&str> {
    let (name, index) = word.strip_suffix(']')?.split_once('[')?;
    (identifier(name) && !index.is_empty() && !index.contains(['[', ']'])).then_some(name)
}

fn forget_unset<'w, 'a: 'w>(
    words: impl Iterator<Item = &'w Word<'a>>,
    arrays: &mut FxHashMap<String, bool>,
) {
    let mut operands = words
        .filter_map(|word| {
            let mut parts = split_quoted_words(word.raw, Whitespace::Ascii)?;
            let operand = parts.pop()?;
            parts.is_empty().then_some(operand)
        })
        .peekable();
    operands.next_if(|operand| operand == "-v");
    operands.next_if(|operand| operand == "--");
    if operands
        .peek()
        .is_some_and(|operand| operand.starts_with('-'))
    {
        return;
    }
    for operand in operands {
        let name = indexed_name(&operand).unwrap_or(&operand);
        if let Some(nonempty) = arrays.get_mut(name) {
            *nonempty = false;
        }
    }
}

fn nounset<'w, 'a: 'w>(mut words: impl Iterator<Item = &'w Word<'a>>, mut active: bool) -> bool {
    while let Some(word) = words.next() {
        let Some((enable, flags)) = flag(word.raw) else {
            break;
        };
        if flags.contains('u') {
            active = enable;
        }
        if flags.contains('o') && words.next().is_some_and(|option| option.raw == "nounset") {
            active = enable;
        }
    }
    active
}

fn flag(word: &str) -> Option<(bool, &str)> {
    let (enable, options) = if let Some(options) = word.strip_prefix('-') {
        (true, options)
    } else {
        (false, word.strip_prefix('+')?)
    };
    (!options.is_empty() && options.bytes().all(|byte| byte.is_ascii_alphabetic()))
        .then_some((enable, options))
}
